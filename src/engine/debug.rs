use std::ffi;

use ash::vk;

pub unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut ffi::c_void,
) -> vk::Bool32 {
    let message = ffi::CStr::from_ptr((*p_callback_data).p_message);
    let severity = format!("{message_severity:?}").to_lowercase();
    let ty = format!("{message_type:?}").to_lowercase();
    println!("[Debug][{severity}][{ty}] {message:?}");
    vk::FALSE
}

pub struct Debug {
    pub loader: ash::extensions::ext::DebugUtils,
    pub messenger: vk::DebugUtilsMessengerEXT,
}

impl Debug {
    pub fn init(entry: &ash::Entry, instance: &ash::Instance) -> Result<Debug, vk::Result> {
        use vk::DebugUtilsMessageSeverityFlagsEXT as SeverityFlags;
        use vk::DebugUtilsMessageTypeFlagsEXT as TypeFlags;
        let mut debug_create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                SeverityFlags::WARNING
                    | SeverityFlags::ERROR
                    | SeverityFlags::INFO
                    | SeverityFlags::VERBOSE,
            )
            .message_type(TypeFlags::VALIDATION | TypeFlags::GENERAL | TypeFlags::PERFORMANCE)
            .pfn_user_callback(Some(vulkan_debug_utils_callback));

        let loader = ash::extensions::ext::DebugUtils::new(entry, instance);
        let messenger = unsafe {
            loader
                .create_debug_utils_messenger(&debug_create_info, None)
                .unwrap()
        };



        Ok(Debug { loader, messenger })
    }
}

impl Drop for Debug {
    fn drop(&mut self) {
        unsafe {
            self.loader
                .destroy_debug_utils_messenger(self.messenger, None)
        };
    }
}
