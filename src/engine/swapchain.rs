use ash::vk;

use super::{
    queue::{QueueFamilies, Queues},
    surface::Surfaces,
};

pub struct SwapChain {
    pub swapchain_loader: ash::extensions::khr::Swapchain,
    pub swapchain: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
    pub framebuffers: Vec<vk::Framebuffer>,
    pub surface_format: vk::SurfaceFormatKHR,
    pub extent: vk::Extent2D,
}

impl SwapChain {
    pub fn init(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        logical_device: &ash::Device,
        surfaces: &Surfaces,
        queue_families: &QueueFamilies,
        queues: &Queues,
    ) -> Result<SwapChain, vk::Result> {
        let surface_capabilities = surfaces.get_capabilities(physical_device).unwrap();
        let surface_format = *surfaces
            .get_formats(physical_device)
            .unwrap()
            .first()
            .unwrap();
        let extent = surface_capabilities.current_extent;
        let queue_families = [queue_families.graphics_queue_index.unwrap()];

        // create swap chains
        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surfaces.surface)
            .min_image_count(match surface_capabilities.max_image_count {
                0 => 3.max(surface_capabilities.min_image_count), // 0 == infinite (see https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkSurfaceCapabilities2EXT.html)
                _ => 3
                    .max(surface_capabilities.min_image_count)
                    .max(surface_capabilities.max_image_count),
            })
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(surface_capabilities.current_extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(&queue_families)
            .pre_transform(surface_capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(vk::PresentModeKHR::FIFO);

        let swapchain_loader = ash::extensions::khr::Swapchain::new(instance, logical_device);
        let swapchain = unsafe {
            swapchain_loader
                .create_swapchain(&swapchain_create_info, None)
                .unwrap()
        };

        let swapchain_images = unsafe { swapchain_loader.get_swapchain_images(swapchain).unwrap() };

        let mut swapchain_image_views = Vec::with_capacity(swapchain_images.len());

        for image in &swapchain_images {
            let subresource_range = vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1);

            let image_view_create_info = vk::ImageViewCreateInfo::builder()
                .image(*image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(vk::Format::B8G8R8A8_SRGB)
                .subresource_range(*subresource_range);
            let image_view = unsafe {
                logical_device
                    .create_image_view(&image_view_create_info, None)
                    .unwrap()
            };
            swapchain_image_views.push(image_view);
        }

        Ok(SwapChain {
            framebuffers: Vec::new(),
            surface_format,
            extent,
            swapchain,
            swapchain_loader,
            image_views: swapchain_image_views,
            images: swapchain_images,
        })
    }

    pub fn create_framebuffers(
        &mut self,
        logical_device: &ash::Device,
        render_pass: vk::RenderPass,
    ) -> Result<(), vk::Result> {
        for iv in &self.image_views {
            let image_view = [*iv];
            let framebuffer_info = vk::FramebufferCreateInfo::builder()
                .render_pass(render_pass)
                .attachments(&image_view)
                .width(self.extent.width)
                .height(self.extent.height)
                .layers(1);
            let fb = unsafe {
                logical_device
                    .create_framebuffer(&framebuffer_info, None)
                    .unwrap()
            };
            self.framebuffers.push(fb);
        }

        Ok(())
    }

    pub unsafe fn cleanup(&mut self, logical_device: &ash::Device) {
        for frame_buffer in &self.framebuffers {
            logical_device.destroy_framebuffer(*frame_buffer, None);
        }
        for image_view in &self.image_views {
            logical_device.destroy_image_view(*image_view, None);
        }
        self.swapchain_loader
            .destroy_swapchain(self.swapchain, None);
    }
}
