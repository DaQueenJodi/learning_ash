use std::{ffi, mem};

use ash::vk;
use vk_shader_macros::include_glsl;

use self::{
    debug::vulkan_debug_utils_callback,
    debug::Debug,
    queue::{QueueFamilies, Queues},
    surface::Surfaces,
    swapchain::SwapChain,
};

pub mod debug;
pub mod device;
pub mod queue;
pub mod surface;
pub mod swapchain;

pub struct GameEngine {
    pub window: winit::window::Window,
    pub entry: ash::Entry,
    pub instance: ash::Instance,
    pub debug: mem::ManuallyDrop<Debug>,
    pub surfaces: mem::ManuallyDrop<Surfaces>,
    pub physical_device: vk::PhysicalDevice,
    pub physical_device_properties: vk::PhysicalDeviceProperties,
    pub queue_families: QueueFamilies,
    pub queues: Queues,
    pub device: ash::Device,
    pub swapchain: SwapChain,
    pub render_pass: vk::RenderPass,
    pub pipeline: Pipeline,
}

impl GameEngine {
    pub fn init(window: winit::window::Window) -> Result<GameEngine, Box<dyn std::error::Error>> {
        let entry = ash::Entry::linked();

        let layer_names = vec!["VK_LAYER_KHRONOS_validation"];
        let instance = init_instance(&entry, &layer_names).unwrap();
        let debug = Debug::init(&entry, &instance).unwrap();
        let surfaces = Surfaces::init(&window, &entry, &instance).unwrap();

        let (physical_device, physical_device_properties) =
            init_physical_devices_and_properties(&instance).unwrap();

        let queue_families = QueueFamilies::init(&instance, physical_device, &surfaces).unwrap();

        let (logical_device, queues) =
            init_devices_and_queues(&instance, physical_device, &queue_families, &layer_names)
                .unwrap();

        let swapchain = SwapChain::init(
            &instance,
            physical_device,
            &logical_device,
            &surfaces,
            &queue_families,
            &queues,
        )
        .unwrap();

        let render_pass = init_render_pass(
            &logical_device,
            physical_device,
            swapchain.surface_format.format,
        )
        .unwrap();

        let pipeline = Pipeline::init(&logical_device, &swapchain, &render_pass).unwrap();

        Ok(GameEngine {
            pipeline,
            window,
            entry,
            instance,
            debug: mem::ManuallyDrop::new(debug),
            surfaces: mem::ManuallyDrop::new(surfaces),
            physical_device,
            physical_device_properties,
            queue_families,
            queues,
            device: logical_device,
            swapchain,
            render_pass,
        })
    }
}

impl Drop for GameEngine {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_render_pass(self.render_pass, None);
            self.swapchain.cleanup(&self.device);
            self.device.destroy_device(None);
            std::mem::ManuallyDrop::drop(&mut self.surfaces);
            std::mem::ManuallyDrop::drop(&mut self.debug);
            self.instance.destroy_instance(None);
        };
    }
}

pub fn init_instance(
    entry: &ash::Entry,
    layer_names: &[&str],
) -> Result<ash::Instance, ash::vk::Result> {
    let app_name = ffi::CString::new("hi :)").unwrap();
    let app_info = vk::ApplicationInfo::builder()
        .application_name(&app_name)
        .application_version(vk::make_api_version(0, 1, 0, 0))
        .api_version(vk::make_api_version(0, 1, 0, 0));

    let layer_names_c: Vec<ffi::CString> = layer_names
        .iter()
        .map(|&ln| ffi::CString::new(ln).unwrap())
        .collect();

    let layer_name_pointers: Vec<*const i8> = layer_names_c.iter().map(|ln| ln.as_ptr()).collect();

    let extension_name_pointers = vec![
        ash::extensions::ext::DebugUtils::name().as_ptr(),
        ash::extensions::khr::Surface::name().as_ptr(),
        ash::extensions::khr::XlibSurface::name().as_ptr(),
    ];

    let mut debug_create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        )
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        )
        .pfn_user_callback(Some(vulkan_debug_utils_callback));

    let instance_create_info = vk::InstanceCreateInfo::builder()
        .push_next(&mut debug_create_info)
        .application_info(&app_info)
        .enabled_layer_names(&layer_name_pointers)
        .enabled_extension_names(&extension_name_pointers);

    unsafe { entry.create_instance(&instance_create_info, None) }
}

pub fn init_physical_devices_and_properties(
    instance: &ash::Instance,
) -> Result<(vk::PhysicalDevice, vk::PhysicalDeviceProperties), vk::Result> {
    let physical_devices = unsafe { instance.enumerate_physical_devices().unwrap() };
    let physical_device = *physical_devices.first().unwrap();
    let properities = unsafe { instance.get_physical_device_properties(physical_device) };
    Ok((physical_device, properities))
}

pub fn init_devices_and_queues(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    queue_families: &QueueFamilies,
    layer_names: &[&str],
) -> Result<(ash::Device, Queues), vk::Result> {
    let layer_names_c: Vec<ffi::CString> = layer_names
        .iter()
        .map(|&ln| ffi::CString::new(ln).unwrap())
        .collect();

    let layer_names_pointers: Vec<*const i8> = layer_names_c
        .iter()
        .map(|layer_name| layer_name.as_ptr())
        .collect();

    let priorities = [1.0];

    let queue_infos = [
        vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_families.graphics_queue_index.unwrap())
            .queue_priorities(&priorities)
            .build(),
        vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_families.transfer_queue_index.unwrap())
            .queue_priorities(&priorities)
            .build(),
    ];

    let device_extension_name_pointers = vec![ash::extensions::khr::Swapchain::name().as_ptr()];
    let device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_infos)
        .enabled_extension_names(&device_extension_name_pointers)
        .enabled_layer_names(&layer_names_pointers);

    let logical_device = unsafe {
        instance
            .create_device(physical_device, &device_create_info, None)
            .unwrap()
    };
    let graphics_queue =
        unsafe { logical_device.get_device_queue(queue_families.graphics_queue_index.unwrap(), 0) };
    let transfer_queue =
        unsafe { logical_device.get_device_queue(queue_families.transfer_queue_index.unwrap(), 0) };

    Ok((
        logical_device,
        Queues {
            graphics_queue,
            transfer_queue,
        },
    ))
}

pub fn init_render_pass(
    logical_device: &ash::Device,
    physical_device: vk::PhysicalDevice,
    format: vk::Format,
) -> Result<vk::RenderPass, vk::Result> {
    let attachments = [vk::AttachmentDescription::builder()
        .format(format)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
        .samples(vk::SampleCountFlags::TYPE_1)
        .build()];

    let color_attachment_references = [vk::AttachmentReference {
        attachment: 0,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
    }];

    let subpasses = [vk::SubpassDescription::builder()
        .color_attachments(&color_attachment_references)
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .build()];

    let subpass_dependencies = [vk::SubpassDependency::builder()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_subpass(0)
        .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_access_mask(
            vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
        )
        .build()];

    let render_pass_info = vk::RenderPassCreateInfo::builder()
        .attachments(&attachments)
        .subpasses(&subpasses)
        .dependencies(&subpass_dependencies);

    unsafe { logical_device.create_render_pass(&render_pass_info, None) }
}

pub struct Pipeline {
    pub pipeline: vk::Pipeline,
    pub layout: vk::PipelineLayout,
}

impl Pipeline {
    pub fn cleanup(&self, logical_device: &ash::Device) {
        unsafe {
            logical_device.destroy_pipeline(self.pipeline, None);
            logical_device.destroy_pipeline_layout(self.layout, None);
        }
    }

    pub fn init(
        logical_device: &ash::Device,
        swapchain: &SwapChain,
        render_pass: &vk::RenderPass,
    ) -> Result<Pipeline, vk::Result> {
        let vertex_shader_create_info =
            vk::ShaderModuleCreateInfo::builder().code(include_glsl!("./shaders/shader.vert"));

        let vertex_shader_module = unsafe {
            logical_device
                .create_shader_module(&vertex_shader_create_info, None)
                .unwrap()
        };

        let entry_point = ffi::CString::new("main").unwrap();

        let vertex_shader_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vertex_shader_module)
            .name(&entry_point);

        let fragment_shader_create_info =
            vk::ShaderModuleCreateInfo::builder().code(include_glsl!("./shaders/shader.frag"));
        let fragment_shader_module = unsafe {
            logical_device
                .create_shader_module(&fragment_shader_create_info, None)
                .unwrap()
        };
        let fragment_shader_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(fragment_shader_module)
            .name(&entry_point);

        let shader_stages = vec![vertex_shader_stage.build(), fragment_shader_stage.build()];

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder();

        let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::POINT_LIST);

        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: swapchain.extent.width as f32,
            height: swapchain.extent.height as f32,
            min_depth: 0.0,
            max_depth: 0.0,
        }];

        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: swapchain.extent,
        }];

        let viewport_info = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&viewports)
            .scissors(&scissors);

        let rasterizing_info = vk::PipelineRasterizationStateCreateInfo::builder()
            .line_width(1.0)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .cull_mode(vk::CullModeFlags::NONE)
            .polygon_mode(vk::PolygonMode::FILL);

        let multisampler_info = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let color_blend_attachments = [vk::PipelineColorBlendAttachmentState::builder()
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .alpha_blend_op(vk::BlendOp::ADD)
            .color_write_mask(
                vk::ColorComponentFlags::R
                    | vk::ColorComponentFlags::G
                    | vk::ColorComponentFlags::A,
            )
            .build()];

        let color_blend_info =
            vk::PipelineColorBlendStateCreateInfo::builder().attachments(&color_blend_attachments);

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder();
        let pipeline_layout = unsafe {
            logical_device
                .create_pipeline_layout(&pipeline_layout_info, None)
                .unwrap()
        };

        let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly_info)
            .viewport_state(&viewport_info)
            .rasterization_state(&rasterizing_info)
            .multisample_state(&multisampler_info)
            .color_blend_state(&color_blend_info)
            .layout(pipeline_layout)
            .render_pass(*render_pass)
            .subpass(0);

        let pipeline = unsafe {
            logical_device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[pipeline_info.build()],
                    None,
                )
                .unwrap()
        }[0];

        unsafe {
            logical_device.destroy_shader_module(vertex_shader_module, None);
            logical_device.destroy_shader_module(fragment_shader_module, None);
        };

        Ok(Pipeline {
            layout: pipeline_layout,
            pipeline,
        })
    }
}
