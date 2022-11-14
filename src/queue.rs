use ash::vk;

use crate::surface::WindowSurface;

pub struct QueueFamilies {
    pub graphics_queue_index: Option<u32>,
    pub transfer_queue_index: Option<u32>,
}

impl QueueFamilies {
    pub fn init(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surface: &WindowSurface,
    ) -> Result<QueueFamilies, vk::Result> {
        let queue_family_properties =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        let mut found_graphics_queue_index = None;
        let mut found_transfer_queue_index = None;

        for (index, queue_family) in queue_family_properties.iter().enumerate() {
            if queue_family.queue_count > 0 {
                if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                    if surface
                        .check_support(physical_device, index as usize)
                        .unwrap()
                    {
                        found_graphics_queue_index = Some(index as u32);
                    }
                } else if queue_family.queue_flags.contains(vk::QueueFlags::TRANSFER) {
                    if found_transfer_queue_index.is_none() // prioritise trasnger queues without graphics
                        || !queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                    {
                        found_transfer_queue_index = Some(index as u32);
                    }
                }
            }
        }
        Ok(QueueFamilies {
            graphics_queue_index: found_graphics_queue_index,
            transfer_queue_index: found_transfer_queue_index,
        })
    }
}
