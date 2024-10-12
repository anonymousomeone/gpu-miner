
use std::sync::Arc;

use vulkano::device::Queue;
use vulkano::device::Device;
use vulkano::buffer::Subbuffer;
use vulkano::sync;
use vulkano::sync::future::NowFuture;
use vulkano::sync::future::FenceSignalFuture;
use vulkano::command_buffer::CommandBufferExecFuture;
use vulkano::command_buffer::PrimaryAutoCommandBuffer;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::sync::GpuFuture;

use super::miner::DISPATCH_SIZE;
use super::MinoeringResult;

pub struct Worker {
    queue: Arc<Queue>,
    device: Arc<Device>,
    input_staging_buffers: Vec<Subbuffer<[u32]>>,
    output_staging_buffers: Vec<Subbuffer<[u32]>>,
    command_buffers: Vec<Arc<PrimaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>>>,
    futures: Vec<FenceSignalFuture<CommandBufferExecFuture<NowFuture>>>,
    max_dispatches: usize,
    dispatch_index: usize,
    nonces: Vec<u64>,
    minoers_mined: usize,
}

impl Worker{
    pub fn new(
        queue: Arc<Queue>,
        device: Arc<Device>,
        input_staging_buffers: Vec<Subbuffer<[u32]>>,
        output_staging_buffers: Vec<Subbuffer<[u32]>>,
        command_buffers: Vec<Arc<PrimaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>>>,
        dispatch_amount: usize,
    ) -> Worker {
        Worker {
            device,
            queue,
            input_staging_buffers,
            output_staging_buffers,
            command_buffers,
            futures: Vec::with_capacity(dispatch_amount),
            max_dispatches: dispatch_amount,
            dispatch_index: 0,
            nonces: Vec::with_capacity(dispatch_amount),
            minoers_mined: 0,
        }
    }

    pub fn submit(&mut self, data: [u32; 10], nonce: u64) {
        if self.dispatch_index >= self.max_dispatches {
            panic!("bruh");
        }

        self.input_staging_buffers[self.dispatch_index].write().unwrap().copy_from_slice(&data);
        // Let's execute this command buffer now.
        let future: FenceSignalFuture<CommandBufferExecFuture<NowFuture>> = sync::now(self.device.clone())
            .then_execute(self.queue.clone(), self.command_buffers[self.dispatch_index].clone())
            .unwrap()
            // This line instructs the GPU to signal a *fence* once the command buffer has finished
            // execution. A fence is a Vulkan object that allows the CPU to know when the GPU has
            // reached a certain point. We need to signal a fence here because below we want to block
            // the CPU until the GPU has reached that point in the execution.
            .then_signal_fence_and_flush()
            .unwrap(); 

        self.dispatch_index += 1;
        self.minoers_mined += 1;
        self.futures.push(future);
        self.nonces.push(nonce);
    }

    pub fn get_results(&mut self) -> Vec<MinoeringResult> {
        let mut output = Vec::new();
        for index in 0..self.max_dispatches {
            self.futures[index].wait(None).unwrap();
            // let end = Instant::now();
            // println!("Elapsed: {}ms", end.duration_since(start).as_millis());
        
            // Now that the GPU is done, the content of the buffer should have been modified. Let's check
            // it out. The call to `read()` would return an error if the buffer was still in use by the
            // GPU.
            let data_buffer_content = self.output_staging_buffers[index].read().unwrap();
    
            for x in 0..(DISPATCH_SIZE*64 / 6 - 1) {
                if data_buffer_content[(x * 6) as usize] == 0x000000 {
                    // if data_buffer_content[(x * 6 + 1) as usize] == 0x000000 {
                    let mut hashes = Vec::new();
                    hashes.push(data_buffer_content[(x * 6) as usize]);
                    hashes.push(data_buffer_content[(x * 6 + 1) as usize]);
                    hashes.push(data_buffer_content[(x * 6 + 2) as usize]);
                    hashes.push(data_buffer_content[(x * 6 + 3) as usize]);
                    hashes.push(data_buffer_content[(x * 6 + 4) as usize]);
    
                    output.push(
                        MinoeringResult::new(data_buffer_content[(x * 6 + 5) as usize] as u64 + self.nonces[index], hashes, self.minoers_mined)
                    );
                    // }
                }
            }
        }
        self.dispatch_index = 0;
        self.futures.clear();
        self.nonces.clear();
        return output;
    }

    pub fn reset(&mut self) {
        self.minoers_mined = 0;
    }

}