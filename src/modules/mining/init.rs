use std::sync::Arc;
use std::vec::IntoIter;

use vulkano::library::VulkanLibrary;
use vulkano::instance::Instance;
use vulkano::instance::InstanceCreateInfo;
use vulkano::device::Queue;
use vulkano::device::QueueCreateInfo;
use vulkano::device::QueueFlags;
use vulkano::device::Device;
use vulkano::device::DeviceCreateInfo;
use vulkano::device::DeviceExtensions;
use vulkano::device::physical::PhysicalDeviceType;

pub fn init_library() -> Arc<Instance> {
    let library = VulkanLibrary::new()
    .unwrap_or_else(|err| panic!("Couldn't load Vulkan library: {:?}", err));

    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            ..Default::default()
        },
    ).unwrap_or_else(|err| panic!("Couldn't create instance: {:?}", err));

    return instance;
}

pub fn init_device(instance: Arc<Instance>) -> (Arc<Device>, IntoIter<Arc<Queue>>) {
    // Choose which physical device to use.
    let device_extensions = DeviceExtensions {
        khr_storage_buffer_storage_class: true,
        ..DeviceExtensions::empty()
    };
    let (physical_device, queue_family_index) = instance
        .enumerate_physical_devices()
        .unwrap()
        .filter(|p| p.supported_extensions().contains(&device_extensions))
        .filter_map(|p| {
            // The Vulkan specs guarantee that a compliant implementation must provide at least one
            // queue that supports compute operations.
            p.queue_family_properties()
                .iter()
                .position(|q| q.queue_flags.intersects(QueueFlags::COMPUTE))
                .map(|i| (p, i as u32))
        })
        .min_by_key(|(p, _)| match p.properties().device_type {
            PhysicalDeviceType::DiscreteGpu => 0,
            PhysicalDeviceType::IntegratedGpu => 1,
            PhysicalDeviceType::VirtualGpu => 2,
            PhysicalDeviceType::Cpu => 3,
            PhysicalDeviceType::Other => 4,
            _ => 5,
        })
        .unwrap();

    // let queue_family_index = 2;
    println!(
        "Using device: {} (type: {:?}); queue family index {}",
        physical_device.properties().device_name,
        physical_device.properties().device_type,
        queue_family_index
    );

    // println!("{:#?}", physical_device.queue_family_properties());
    let mut queue_priorities = vec![0.5];
    queue_priorities.resize(
        physical_device.queue_family_properties()[queue_family_index as usize].queue_count as usize,
        0.5
    );

    // Now initializing the device.
    let (device, queues) = Device::new(
        physical_device,
        DeviceCreateInfo {
            enabled_extensions: device_extensions,
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                queues: queue_priorities,
                ..Default::default()
            }],
            ..Default::default()
        },
    )
    .unwrap();

    return (device, queues.collect::<Vec<Arc<Queue>>>().into_iter())
}