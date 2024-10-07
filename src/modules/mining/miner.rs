use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread;

use vulkano::buffer::Buffer;
use vulkano::buffer::Subbuffer;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::BufferCreateInfo;
use vulkano::command_buffer::CommandBufferUsage;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::CommandBufferExecFuture;
use vulkano::command_buffer::CopyBufferInfo;
use vulkano::command_buffer::PrimaryAutoCommandBuffer;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::descriptor_set::WriteDescriptorSet;
use vulkano::descriptor_set::PersistentDescriptorSet;
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::device::Queue;
use vulkano::device::Device;
use vulkano::memory::allocator::MemoryTypeFilter;
use vulkano::memory::allocator::AllocationCreateInfo;
use vulkano::memory::allocator::StandardMemoryAllocator;

use vulkano::pipeline::Pipeline;
use vulkano::pipeline::PipelineLayout;
use vulkano::pipeline::PipelineBindPoint;
use vulkano::pipeline::PipelineShaderStageCreateInfo;
use vulkano::pipeline::compute::ComputePipeline;
use vulkano::pipeline::compute::ComputePipelineCreateInfo;
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::sync;
use vulkano::sync::GpuFuture;
use vulkano::sync::future::NowFuture;
use vulkano::sync::future::FenceSignalFuture;

use super::MinoeringResult;


pub struct Minoer {
    device: Arc<Device>,
    queues: Vec<Arc<Queue>>,

    data_reciever: Receiver<[u32; 10]>,
    result_sender: Sender<MinoeringResult>,

    pub dispatch_index: usize,
    max_dispatches: usize,
    queue_amount: usize,
    input_staging_buffers: Vec<Subbuffer<[u32]>>,
    output_staging_buffers: Vec<Subbuffer<[u32]>>,
    command_buffers: Vec<Arc<PrimaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>>>,
    futures: Vec<FenceSignalFuture<CommandBufferExecFuture<NowFuture>>>,
    nonces: Vec<u64>,
}

pub const DISPATCH_SIZE: u32 = 65536;

impl Minoer {
    pub fn new(max_dispatches: usize, data_reciever: Receiver<[u32; 10]>, result_sender: Sender<MinoeringResult>) -> Minoer {
        let instance = crate::modules::mining::init::init_library();

        let (device, queues) = crate::modules::mining::init::init_device(instance);
    
        let ocs = crate::modules::mining::shader::cs::load(device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();
        let stage = PipelineShaderStageCreateInfo::new(ocs);
        let layout = PipelineLayout::new(
            device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages([&stage])
                .into_pipeline_layout_create_info(device.clone())
                .unwrap(),
        )
        .unwrap();
        let pipeline = ComputePipeline::new(
            device.clone(),
            None,
            ComputePipelineCreateInfo::stage_layout(stage, layout),
        )
        .unwrap();
    
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ));
        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));

        let mut input_staging_buffers = Vec::new();
        let mut output_staging_buffers = Vec::new();
        let mut command_buffers = Vec::new();
        let futures = Vec::new();

        for _ in 0..queues.len() {
            let input_staging_buffer = Buffer::new_slice::<u32>(
                memory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::TRANSFER_SRC,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::HOST_SEQUENTIAL_WRITE
                        | MemoryTypeFilter::PREFER_HOST,
                    ..Default::default()
                },
                10,
            ).unwrap();

            // Create a buffer in device-local memory.
            let input_buffer = Buffer::new_slice::<u32>(
                memory_allocator.clone(),
                BufferCreateInfo {
                    // Specify use as a storage buffer and transfer destination.
                    usage: BufferUsage::STORAGE_BUFFER | BufferUsage::TRANSFER_DST,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    // Specify use by the device only.
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                    ..Default::default()
                },
                10,
            )
            .unwrap();

            let output_staging_buffer = Buffer::new_slice::<u32>(
                memory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::TRANSFER_DST,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::HOST_SEQUENTIAL_WRITE
                        | MemoryTypeFilter::PREFER_HOST,
                    ..Default::default()
                },
                (DISPATCH_SIZE*64).into()
            ).unwrap();

            // Create a buffer in device-local memory.
            let output_buffer = Buffer::new_slice::<u32>(
                memory_allocator.clone(),
                BufferCreateInfo {
                    // Specify use as a storage buffer and transfer source.
                    usage: BufferUsage::STORAGE_BUFFER | BufferUsage::TRANSFER_SRC,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    // Specify use by the device only.
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                    ..Default::default()
                },
                (DISPATCH_SIZE*64).into(),
            )
            .unwrap();

    
            // In order to let the shader access the buffer, we need to build a *descriptor set* that
            // contains the buffer.
            //
            // The resources that we bind to the descriptor set must match the resources expected by the
            // pipeline which we pass as the first parameter.
            //
            // If you want to run the pipeline on multiple different buffers, you need to create multiple
            // descriptor sets that each contain the buffer you want to run the shader on.
            let layout = &pipeline.layout().set_layouts()[0];
            let set = PersistentDescriptorSet::new(
                &descriptor_set_allocator,
                layout.clone(),
                [
                    WriteDescriptorSet::buffer(0, input_buffer.clone()),
                    WriteDescriptorSet::buffer(1, output_buffer.clone())
                ],
                [],
            )
            .unwrap();

            // In order to execute our operation, we have to build a command buffer.
            let mut builder = AutoCommandBufferBuilder::primary(
                &command_buffer_allocator, 
                queues[0].queue_family_index(), 
                CommandBufferUsage::MultipleSubmit,
            ).unwrap();
    
            // Note that we clone the pipeline and the set. Since they are both wrapped in an `Arc`,
            // this only clones the `Arc` and not the whole pipeline or set (which aren't cloneable
            // anyway). In this example we would avoid cloning them since this is the last time we use
            // them, but in real code you would probably need to clone them.
            builder
                .copy_buffer(CopyBufferInfo::buffers(
                    input_staging_buffer.clone(),
                    input_buffer
                )).unwrap()
                .bind_pipeline_compute(
                    pipeline.clone()
                ).unwrap()
                .bind_descriptor_sets(
                    PipelineBindPoint::Compute,
                    pipeline.layout().clone(),
                    0,
                    set,
                ).unwrap()
                .dispatch([DISPATCH_SIZE, 1, 1]).unwrap()
                .copy_buffer(CopyBufferInfo::buffers(
                    output_buffer,
                    output_staging_buffer.clone()
                )).unwrap();
        
            // Finish building the command buffer by calling `build`.
            let command_buffer = builder.build().unwrap();

            input_staging_buffers.push(input_staging_buffer);
            output_staging_buffers.push(output_staging_buffer);
            command_buffers.push(command_buffer);
        }

        let queue_amount = queues.len();
        Minoer {
            device,
            queues,

            data_reciever,
            result_sender,

            dispatch_index: 0,
            max_dispatches,
            queue_amount,
            input_staging_buffers,
            output_staging_buffers,
            command_buffers,
            futures,
            nonces: Vec::new(),
        }
    }

    pub fn mine(&mut self, data: [u32; 10], nonce: u64) {
        if self.dispatch_index >= self.max_dispatches {
            panic!("bruh");
        }

        for i in 0..self.queue_amount {
            let input_buffer = self.input_staging_buffers.get(self.queue_amount).unwrap().clone();
            let command_buffer = self.command_buffers.get(self.queue_amount).unwrap().clone();
            thread::spawn(move || {
                input_buffer.write().unwrap().copy_from_slice(&data);
                // Let's execute this command buffer now.
                let future: FenceSignalFuture<CommandBufferExecFuture<NowFuture>> = sync::now(self.device.clone())
                    .then_execute(self.queues[self.dispatch_index % self.queue_amount].clone(), command_buffer)
                    .unwrap()
                    // This line instructs the GPU to signal a *fence* once the command buffer has finished
                    // execution. A fence is a Vulkan object that allows the CPU to know when the GPU has
                    // reached a certain point. We need to signal a fence here because below we want to block
                    // the CPU until the GPU has reached that point in the execution.
                    .then_signal_fence_and_flush()
                    .unwrap(); 
            });
        }
        // let mut output: Vec<u32> = Vec::new();
        // let mut data: [u32; 6] = [0, 0, 0, 0, 0, 0];
        // let hash = String::from("abc");
        // let nonce: u32 = 0b10000000000000000000000000000000;
    
        // data[0] = to_u32(hex::decode(&hash[0..8]).unwrap());
        // data[1] = to_u32(hex::decode(&hash[8..16]).unwrap());
        // data[2] = to_u32(hex::decode(&hash[16..24]).unwrap());
        // data[3] = to_u32(hex::decode(&hash[24..32]).unwrap());
        // data[4] = to_u32(hex::decode(&hash[32..40]).unwrap());
        // data[5] = to_u32(hex::decode("80000000").unwrap());
        self.nonces.push(nonce);
        self.dispatch_index += 1;

        // println!();
    
        // println!("Success");

    }

    pub fn get_results(&mut self, sender: &Sender<MinoeringResult>) {

        for i in 0..self.max_dispatches {
            let output_buffer = self.output_staging_buffers.get(i).unwrap();
            let future = self.futures.get(i).unwrap();
            let nonce = self.nonces.get(i).unwrap();
            // Blocks execution until the GPU has finished the operation. This method only exists on the
            // future that corresponds to a signalled fence. In other words, this method wouldn't be
            // available if we didn't call `.then_signal_fence_and_flush()` earlier. The `None` parameter
            // is an optional timeout.
            //
            // Note however that dropping the `future` variable (with `drop(future)` for example) would
            // block execution as well, and this would be the case even if we didn't call
            // `.then_signal_fence_and_flush()`. Therefore the actual point of calling
            // `.then_signal_fence_and_flush()` and `.wait()` is to make things more explicit. In the
            // future, if the Rust language gets linear types vulkano may get modified so that only
            // fence-signalled futures can get destroyed like this.
            // let start = Instant::now();
            thread::scope(move |scope| {
                future.wait(None).unwrap();
                // let end = Instant::now();
                // println!("Elapsed: {}ms", end.duration_since(start).as_millis());
            
                // Now that the GPU is done, the content of the buffer should have been modified. Let's check
                // it out. The call to `read()` would return an error if the buffer was still in use by the
                // GPU.
                let data_buffer_content = output_buffer.read().unwrap();

                scope.spawn(move || {
                    for x in 0..(DISPATCH_SIZE*64 / 6 - 1) {
                        if data_buffer_content[(x * 6) as usize] == 0x000000 {
                            // if data_buffer_content[(x * 6 + 1) as usize] == 0x000000 {
                            let mut hashes = Vec::new();
                            hashes.push(data_buffer_content[(x * 6) as usize]);
                            hashes.push(data_buffer_content[(x * 6 + 1) as usize]);
                            hashes.push(data_buffer_content[(x * 6 + 2) as usize]);
                            hashes.push(data_buffer_content[(x * 6 + 3) as usize]);
                            hashes.push(data_buffer_content[(x * 6 + 4) as usize]);
        
                            sender.send(
                                MinoeringResult::new(data_buffer_content[(x * 6 + 5) as usize] as u64 + nonce, hashes)
                            ).unwrap();
                            // }
                        }
                        // println!();
                    }
                });
            });
        }

        self.futures.clear();
        self.nonces.clear();
        self.dispatch_index = 0;
    }
}

