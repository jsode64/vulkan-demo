use ash::vk::Handle;
use ash::{Device, Entry, Instance, ext::debug_utils, khr::surface, vk};
use core::slice;
use std::ops::Deref;
use std::sync::Arc;
use std::{
    ffi::{CStr, c_void},
    ptr,
};

use crate::{Error, Result, VERSION, Version, Window, vk_query};

/// Queue family indices.
struct QueueFamilyIndices {
    /// The main queue.
    main_index: u32,

    /// The present queue index.
    present_index: u32,
}

/// A builder for a context.
pub struct ContextBuilder<'a> {
    /// The application name.
    app_name: &'a CStr,

    /// The application version.
    app_version: Version,

    /// Whether to enable validation layers.
    use_validation: bool,
}

/// A Vulkan context.
pub struct ContextInner {
    /// The Vulkan entry.
    entry: Entry,

    /// The window.
    window: *const Window,

    /// The Vulkan instance.
    instance: Instance,

    /// The debug utility instance.
    debug_utils_instance: Option<debug_utils::Instance>,

    /// The window surface instance.
    surface_instance: surface::Instance,

    /// The debug utility messenger.
    debug_messenger: vk::DebugUtilsMessengerEXT,

    /// The window surface.
    surface: vk::SurfaceKHR,

    /// The physical device.
    physical_device: vk::PhysicalDevice,

    /// The device memory properties.
    memory_properties: vk::PhysicalDeviceMemoryProperties,

    /// The device.
    device: Device,

    /// The queue family indices.
    queue_family_indices: QueueFamilyIndices,

    /// The main queue.
    main_queue: vk::Queue,

    /// The present queue.
    present_queue: vk::Queue,

    /// The transfer command pool.
    transfer_pool: vk::CommandPool,
}

/// A Vulkan context.
///
/// This structure contains an `Arc` handle to a structure containing:
/// - The Ash entry
/// - The Vulkan instance
/// - The debug instance and messenger (if enabled)
/// - The surface instance and handle
/// - The physical device (and its memory properties)
/// - The queue indices
/// - The logical device
/// - The queues
///
/// The context will provide the following queues:
/// - A main queue (has compute, graphics, and transfer capabilities)
/// - A present queue
///
/// Getters are present for every item except the debug values.
/// These are made for library internals but they can be used at your own risk.
/// The library can not guarantee you won't, for example, destroy the device.
#[derive(Clone)]
pub struct Context(Arc<ContextInner>);

impl<'a> ContextBuilder<'a> {
    /// Builds the window.
    #[inline]
    #[must_use]
    pub fn build(self, window: &Window) -> Result<Context> {
        Ok(Context(Arc::new(ContextInner::new(self, window)?)))
    }

    /// Sets the application name.
    #[inline]
    #[must_use]
    pub const fn app_name(mut self, app_name: &'a CStr) -> Self {
        self.app_name = app_name;
        self
    }

    /// Sets the application version.
    #[inline]
    #[must_use]
    pub const fn app_version(mut self, app_version: Version) -> Self {
        self.app_version = app_version;
        self
    }

    /// Sets whether validation layers should be enabled.
    #[inline]
    #[must_use]
    pub const fn use_validation(mut self, use_validation: bool) -> Self {
        self.use_validation = use_validation;
        self
    }
}

impl ContextInner {
    /// Creates a window from the builder and window.
    #[must_use]
    pub fn new(builder: ContextBuilder, window: &Window) -> Result<Self> {
        let entry = unsafe { Entry::load().unwrap() };
        let instance = create_instance(&entry, &builder, window)?;
        let debug_utils_instance = builder
            .use_validation
            .then(|| debug_utils::Instance::new(&entry, &instance));
        let surface_instance = surface::Instance::new(&entry, &instance);
        let surface = create_surface(&instance, &window)?;
        let debug_messenger = if let Some(debug_utils_instance) = debug_utils_instance.as_ref() {
            create_debug_messenger(debug_utils_instance)?
        } else {
            vk::DebugUtilsMessengerEXT::null()
        };
        let (physical_device, queue_family_indices) =
            choose_physical_device(&instance, &surface_instance, surface)?;
        let memory_properties =
            unsafe { instance.get_physical_device_memory_properties(physical_device) };
        let device = create_device(&instance, physical_device, &queue_family_indices)?;

        let main_queue = unsafe { device.get_device_queue(queue_family_indices.main_index, 0) };
        let present_queue =
            unsafe { device.get_device_queue(queue_family_indices.present_index, 0) };
        let transfer_pool_create_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::empty())
            .queue_family_index(queue_family_indices.main_index);
        let transfer_pool =
            unsafe { vk_query!(device.create_command_pool(&transfer_pool_create_info, None))? };

        Ok(Self {
            entry,
            window: ptr::from_ref(window),
            instance,
            debug_utils_instance,
            surface_instance,
            debug_messenger,
            surface,
            physical_device,
            memory_properties,
            queue_family_indices,
            device,
            main_queue,
            present_queue,
            transfer_pool,
        })
    }

    /// Returns a temporary command buffer.
    #[must_use]
    pub fn temporary_command_buffer(&self) -> Result<vk::CommandBuffer> {
        let allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(self.transfer_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        let transfer_command_buffer =
            unsafe { vk_query!(self.device.allocate_command_buffers(&allocate_info))?[0] };

        Ok(transfer_command_buffer)
    }

    /// Returns the Ash entry.
    #[inline]
    #[must_use]
    pub fn entry(&self) -> &Entry {
        &self.entry
    }

    /// Returns the Vulkan instance.
    #[inline]
    #[must_use]
    pub fn instance(&self) -> &Instance {
        &self.instance
    }

    /// Returns the window.
    #[inline]
    #[must_use]
    pub fn window(&self) -> &Window {
        unsafe { &*self.window }
    }

    /// Returns the context's surface.
    #[inline]
    #[must_use]
    pub fn surface(&self) -> vk::SurfaceKHR {
        self.surface
    }

    /// Returns the physical device being used.
    #[inline]
    #[must_use]
    pub fn physical_device(&self) -> vk::PhysicalDevice {
        self.physical_device
    }

    /// Returns the physical device's memory properties.
    #[inline]
    #[must_use]
    pub fn memory_properties(&self) -> &vk::PhysicalDeviceMemoryProperties {
        &self.memory_properties
    }

    /// Returns the device.
    #[inline]
    #[must_use]
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Returns the window surface instance.
    #[inline]
    #[must_use]
    pub(crate) fn surface_instance(&self) -> &surface::Instance {
        &self.surface_instance
    }

    /// Returns the main queue index.
    #[inline]
    #[must_use]
    pub fn main_queue_index(&self) -> u32 {
        self.queue_family_indices.main_index
    }

    /// Returns the present queue index.
    #[inline]
    #[must_use]
    pub fn present_queue_index(&self) -> u32 {
        self.queue_family_indices.present_index
    }

    /// Returns the main queue.
    #[inline]
    #[must_use]
    pub fn main_queue(&self) -> vk::Queue {
        self.main_queue
    }

    /// Returns the present queue.
    #[inline]
    #[must_use]
    pub fn present_queue(&self) -> vk::Queue {
        self.present_queue
    }

    /// Returns the transfer command pool handle.
    #[inline]
    #[must_use]
    pub fn transfer_pool(&self) -> vk::CommandPool {
        self.transfer_pool
    }
}

impl Drop for ContextInner {
    fn drop(&mut self) {
        let _ = unsafe { self.device.device_wait_idle() };

        unsafe {
            self.device.destroy_command_pool(self.transfer_pool, None);

            self.device.destroy_device(None);
            self.surface_instance.destroy_surface(self.surface, None);

            if let Some(debug_utils_instance) = self.debug_utils_instance.as_ref() {
                debug_utils_instance.destroy_debug_utils_messenger(self.debug_messenger, None);
            }

            self.instance.destroy_instance(None);
        }
    }
}

impl Context {
    /// Returns a default context builder.
    ///
    /// The default values are:
    /// - `app_name`: empty string
    /// - `app_version`: v0.0.0
    /// - `use_validation`: set to `cfg!(debug_assertions)`
    #[inline]
    #[must_use]
    pub const fn builder<'a>() -> ContextBuilder<'a> {
        ContextBuilder {
            app_name: c"",
            app_version: Version::new(0, 0, 0),
            use_validation: cfg!(debug_assertions),
        }
    }
}

impl Deref for Context {
    type Target = Arc<ContextInner>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Creates a new Vulkan instance from the builder.
#[must_use]
fn create_instance(entry: &Entry, builder: &ContextBuilder, _window: &Window) -> Result<Instance> {
    // Get extensions.
    let mut extensions = Vec::new();
    let mut num_required_extensions = 0;
    let required_extensions = unsafe {
        glfw::ffi::glfwGetRequiredInstanceExtensions(ptr::from_mut(&mut num_required_extensions))
    };
    let required_extenisons =
        unsafe { slice::from_raw_parts(required_extensions, num_required_extensions as usize) };
    for &required_extension in required_extenisons {
        extensions.push(required_extension);
    }
    if builder.use_validation {
        extensions.push(debug_utils::NAME.as_ptr());
    }

    // Get layers.
    let mut layers = Vec::new();
    if builder.use_validation {
        layers.push(c"VK_LAYER_KHRONOS_validation".as_ptr());
    }

    let app_version = vk::make_api_version(
        0,
        builder.app_version.major as u32,
        builder.app_version.minor as u32,
        builder.app_version.patch as u32,
    );
    let engine_version = vk::make_api_version(
        0,
        VERSION.major as u32,
        VERSION.minor as u32,
        VERSION.patch as u32,
    );
    let app_info = vk::ApplicationInfo::default()
        .application_name(builder.app_name)
        .application_version(app_version)
        .engine_name(c"Ibis")
        .engine_version(engine_version)
        .api_version(vk::API_VERSION_1_3);
    let create_info = vk::InstanceCreateInfo::default()
        .application_info(&app_info)
        .enabled_layer_names(&layers)
        .enabled_extension_names(&extensions);
    let instance = unsafe {
        entry
            .create_instance(&create_info, None)
            .map_err(|e| Error::VkError(e))?
    };

    Ok(instance)
}

/// Creates a debug messenger.
#[must_use]
fn create_debug_messenger(instance: &debug_utils::Instance) -> Result<vk::DebugUtilsMessengerEXT> {
    let message_severity = vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
        | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
        | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR;
    let message_type = vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
        | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
        | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE;
    let create_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
        .message_severity(message_severity)
        .message_type(message_type)
        .pfn_user_callback(Some(debug_callback));
    let debug_messenger = unsafe {
        instance
            .create_debug_utils_messenger(&create_info, None)
            .map_err(|e| Error::VkError(e))?
    };

    Ok(debug_messenger)
}

/// Vulkan debug messenger callback.
unsafe extern "system" fn debug_callback(
    _message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    _message_types: vk::DebugUtilsMessageTypeFlagsEXT,
    callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
    _user_data: *mut c_void,
) -> vk::Bool32 {
    let message = unsafe { CStr::from_ptr((*callback_data).p_message) };
    eprintln!("{}", message.to_string_lossy());

    vk::FALSE
}

/// Creates a window surface.
#[must_use]
fn create_surface(instance: &Instance, window: &Window) -> Result<vk::SurfaceKHR> {
    let mut surface = vk::SurfaceKHR::null();
    let result = unsafe {
        window.window().create_window_surface(
            instance.handle().as_raw() as _,
            ptr::null(),
            ptr::from_mut(&mut surface) as *mut _,
        )
    };

    (result == 0)
        .then_some(surface)
        .ok_or_else(|| Error::VkError(vk::Result::from_raw(result)))
}

/// Chooses a physical device and returns the best match.
#[must_use]
fn choose_physical_device(
    instance: &Instance,
    surface_instance: &surface::Instance,
    surface: vk::SurfaceKHR,
) -> Result<(vk::PhysicalDevice, QueueFamilyIndices)> {
    let physical_devices = unsafe {
        instance
            .enumerate_physical_devices()
            .map_err(|e| Error::VkError(e))?
    };

    let mut best = None;
    let mut best_score = 0;
    for physical_device in physical_devices {
        let Some(queue_family_indices) =
            get_queue_family_indices(instance, surface_instance, physical_device, surface)
        else {
            continue;
        };

        let score = grade_physical_device(instance, physical_device)?;
        if let Some(score) = score
            && score >= best_score
        {
            best_score = score;
            best = Some((physical_device, queue_family_indices));
        }
    }

    best.ok_or_else(|| Error::NoSuitablePhysicalDevice)
}

/// Scores a physical device.
#[must_use]
fn grade_physical_device(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
) -> Result<Option<u32>> {
    let properties = unsafe { instance.get_physical_device_properties(physical_device) };

    // Make sure the physical device has swapchain support.
    let extensions = unsafe {
        instance
            .enumerate_device_extension_properties(physical_device)
            .map_err(|e| Error::VkError(e))?
    };
    let has_swapchain = extensions.iter().any(|extension| {
        let name = unsafe { CStr::from_ptr(extension.extension_name.as_ptr()) };
        name == c"VK_KHR_swapchain"
    });
    if !has_swapchain {
        return Ok(None);
    }

    // Grade the physical device.
    let mut score = 1;
    if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
        score += 1 << 30;
    }

    Ok(Some(score))
}

/// Returns queue family indices for the physical device.
#[must_use]
fn get_queue_family_indices(
    instance: &Instance,
    surface_instance: &surface::Instance,
    physical_device: vk::PhysicalDevice,
    surface: vk::SurfaceKHR,
) -> Option<QueueFamilyIndices> {
    let queue_families =
        unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
    let mut main_index = None;
    let mut present_index = None;

    for (i, queue_family) in queue_families.iter().enumerate() {
        let i = i as u32;

        let main_flags =
            vk::QueueFlags::GRAPHICS | vk::QueueFlags::COMPUTE | vk::QueueFlags::TRANSFER;
        if main_index.is_none() && queue_family.queue_flags.contains(main_flags) {
            main_index = Some(i);
        }

        if present_index.is_none() {
            let supported = unsafe {
                surface_instance
                    .get_physical_device_surface_support(physical_device, i, surface)
                    .ok()?
            };
            if supported {
                present_index = Some(i);
            }
        }
    }

    Some(QueueFamilyIndices {
        main_index: main_index?,
        present_index: present_index?,
    })
}

/// Creates the logical device and queues.
#[must_use]
fn create_device(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    queue_family_indices: &QueueFamilyIndices,
) -> Result<Device> {
    let queue_priorities = [1.0];
    let queue_infos = [
        vk::DeviceQueueCreateInfo::default()
            .flags(vk::DeviceQueueCreateFlags::empty())
            .queue_family_index(queue_family_indices.main_index)
            .queue_priorities(&queue_priorities),
        vk::DeviceQueueCreateInfo::default()
            .flags(vk::DeviceQueueCreateFlags::empty())
            .queue_family_index(queue_family_indices.present_index)
            .queue_priorities(&queue_priorities),
    ];
    let num_queues = if queue_family_indices.main_index == queue_family_indices.present_index {
        1
    } else {
        2
    };

    let core_features = vk::PhysicalDeviceFeatures::default()
        .geometry_shader(true)
        .tessellation_shader(true);
    let mut features13 = vk::PhysicalDeviceVulkan13Features::default()
        .synchronization2(true)
        .maintenance4(true)
        .dynamic_rendering(true);
    let extensions = vec![vk::KHR_SWAPCHAIN_NAME.as_ptr()];
    let create_info = vk::DeviceCreateInfo::default()
        .queue_create_infos(&queue_infos[..num_queues])
        .enabled_extension_names(&extensions)
        .enabled_features(&core_features)
        .push_next(&mut features13);
    let device = unsafe {
        instance
            .create_device(physical_device, &create_info, None)
            .map_err(|e| Error::VkError(e))?
    };

    Ok(device)
}
