use std::ops::Range;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread;

use vulkano::buffer::Buffer;
use vulkano::buffer::Subbuffer;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::BufferCreateInfo;
use vulkano::command_buffer::CopyBufferInfo;
use vulkano::command_buffer::CommandBufferUsage;
use vulkano::command_buffer::AutoCommandBufferBuilder;
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

use crate::modules::helpers;
use super::worker::Worker;
use super::MinoerControlType;
use super::MinoeringResult;

pub struct Minoer {
    control_senders: Vec<Sender<MinoerControlType>>,
    threads: usize,
}

pub const DISPATCH_SIZE: u32 = 65536;

impl Minoer {
    pub fn new(max_dispatches: usize, result_sender: Sender<MinoeringResult>) -> Minoer {
        let instance = crate::modules::mining::init::init_library();

        let (device, mut queues) = crate::modules::mining::init::init_device(instance);
    
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

        let mut control_senders = Vec::new();
        let queue_amount = queues.len();
        
        for _ in 0..queue_amount {
            let (control_sender, control_receiver) = mpsc::channel::<MinoerControlType>();
            let mut input_staging_buffers = Vec::new();
            let mut output_staging_buffers = Vec::new();
            let mut command_buffers = Vec::new();
            let queue = queues.next().unwrap();
            
            for _ in 0..max_dispatches {

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
                        memory_type_filter: MemoryTypeFilter::HOST_RANDOM_ACCESS
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
                    queue.queue_family_index(), 
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
            control_senders.push(control_sender);

            Minoer::spawn_thread(
                device.clone(),
                queue,
                input_staging_buffers,
                output_staging_buffers,
                command_buffers,
                control_receiver,
                result_sender.clone(),
            );
        }
        // println!("{}", queues.len());

        let minoer = Minoer {
            control_senders,
            threads: queue_amount,
        };

        return minoer;
    }

    pub fn mine(&mut self, data: [u32; 10], nonce: u64) {
        for i in 0..self.threads {
            let range: u64 = (i*99999).try_into().unwrap();
            let range_top: u64 = ((i+1)*99999).try_into().unwrap();

            self.control_senders[i].send(
                MinoerControlType::Start(data, nonce, range..range_top)
            ).unwrap();
        }
    }

    pub fn stop_mining(&self) {
        for i in 0..self.threads {
            self.control_senders[i].send(
                MinoerControlType::Stop
            ).unwrap();
        }
    }

    fn spawn_thread(
        device: Arc<Device>,
        queue: Arc<Queue>,
        input_staging_buffers: Vec<Subbuffer<[u32]>>,
        output_staging_buffers: Vec<Subbuffer<[u32]>>,
        command_buffers: Vec<Arc<PrimaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>>>,
        control_reciever: Receiver<MinoerControlType>,
        result_sender: Sender<MinoeringResult>,
    ) {
        thread::spawn(move || {
            let max_dispatches = command_buffers.len();
            let mut worker = Worker::new(queue, device, input_staging_buffers, output_staging_buffers, command_buffers, max_dispatches);
            let mut data: [u32; 10];
            let mut nonce: u64;
            let mut range: Range<u64>;

            loop {
                (data, nonce, range) = match control_reciever.recv() {
                    Ok(d) => match d {
                        MinoerControlType::Stop => continue,
                        MinoerControlType::Start(d, n, r) => {
                            worker.reset();
                            (d, n, r)
                        },
                    },
                    Err(_) => break,
                };
                
                for i in range {
                    match control_reciever.try_recv() {
                        Ok(d) => match d {
                            MinoerControlType::Stop => break,
                            MinoerControlType::Start(..) => {},
                        },
                        Err(e) => match e {
                            mpsc::TryRecvError::Empty => {},
                            mpsc::TryRecvError::Disconnected => return,
                        },
                    };
    
                    let nonce: u64 = nonce + i * 10_u64.pow(10);
                    let nonce_arr = helpers::nonce_to_u32arr(nonce);
                    data[5] = nonce_arr[0];
                    data[6] = nonce_arr[1];
                    data[7] = nonce_arr[2];
                    data[8] = nonce_arr[3];
                    data[9] = nonce_arr[4];

                    worker.submit(data, nonce);
                    if (i + 1) % max_dispatches as u64 == 0 && (i != 0 || max_dispatches == 1) {
                        let results = worker.get_results();

                        for result in results {
                            result_sender.send(result).unwrap();
                        }
                    }
                }
            }
        });
    }
}

