use ash::{Device, khr::swapchain, vk};

use crate::{Context, Error, Result, Window};

/// A swapchain's image.
#[derive(Clone, Default)]
pub struct SwapchainImage {
    /// The image.
    image: vk::Image,

    /// The image view.
    image_view: vk::ImageView,
}

/// A swapchain state.
pub struct Swapchain {
    /// The swapchain device.
    swapchain_device: swapchain::Device,

    /// The swapchain.
    swapchain: vk::SwapchainKHR,

    /// The swapchain images.
    images: Box<[SwapchainImage]>,

    /// The swapchain's extent.
    extent: vk::Extent2D,

    /// The surface's format.
    surface_format: vk::SurfaceFormatKHR,
}

impl SwapchainImage {
    /// Returns the image.
    #[inline]
    #[must_use]
    pub fn image(&self) -> vk::Image {
        self.image
    }

    /// Returns the image view.
    #[inline]
    #[must_use]
    pub fn image_view(&self) -> vk::ImageView {
        self.image_view
    }
}

impl Swapchain {
    /// Creates a swapchain.
    pub fn new(context: &Context, previous: vk::SwapchainKHR, num_images: u32) -> Result<Self> {
        let surface_instance = context.surface_instance();

        let capabilities = unsafe {
            surface_instance
                .get_physical_device_surface_capabilities(
                    context.physical_device(),
                    context.surface(),
                )
                .map_err(|e| Error::VkError(e))?
        };
        let extent = Self::get_extent(context.window(), &capabilities);
        let surface_format = Self::best_surface_format(context)?;
        let present_mode = Self::best_present_mode(context)?;
        let num_images = Self::get_num_images(&capabilities, num_images);

        // Create swapchain device and swapchain.
        let image_sharing_mode = if context.main_queue_index() == context.present_queue_index() {
            vk::SharingMode::EXCLUSIVE
        } else {
            vk::SharingMode::CONCURRENT
        };
        let queue_family_indices = [context.main_queue_index(), context.present_queue_index()];
        let create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(context.surface())
            .min_image_count(num_images)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(image_sharing_mode)
            .queue_family_indices(&queue_family_indices)
            .pre_transform(capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .old_swapchain(previous);
        let swapchain_device = swapchain::Device::new(context.instance(), context.device());
        let swapchain = unsafe {
            swapchain_device
                .create_swapchain(&create_info, None)
                .map_err(|e| Error::VkError(e))?
        };

        // Create swapchain image objects.
        let swapchain_images = unsafe {
            swapchain_device
                .get_swapchain_images(swapchain)
                .map_err(|e| Error::VkError(e))?
        };
        let mut images = vec![SwapchainImage::default(); swapchain_images.len()].into_boxed_slice();
        for (i, image) in images.iter_mut().enumerate() {
            let component_mapping = vk::ComponentMapping::default()
                .r(vk::ComponentSwizzle::IDENTITY)
                .g(vk::ComponentSwizzle::IDENTITY)
                .b(vk::ComponentSwizzle::IDENTITY)
                .a(vk::ComponentSwizzle::IDENTITY);
            let subresource_range = vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1);
            let create_info = vk::ImageViewCreateInfo::default()
                .image(swapchain_images[i])
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(surface_format.format)
                .components(component_mapping)
                .subresource_range(subresource_range);
            let image_view = unsafe {
                context
                    .device()
                    .create_image_view(&create_info, None)
                    .map_err(|e| Error::VkError(e))?
            };

            image.image = swapchain_images[i];
            image.image_view = image_view;
        }

        Ok(Self {
            swapchain_device,
            swapchain,
            images,
            extent,
            surface_format,
        })
    }

    /// Returns the best supported surface format.
    #[must_use]
    fn best_surface_format(context: &Context) -> Result<vk::SurfaceFormatKHR> {
        let formats = unsafe {
            context
                .surface_instance()
                .get_physical_device_surface_formats(context.physical_device(), context.surface())
                .map_err(|e| Error::VkError(e))?
        };
        let preferred_format = vk::SurfaceFormatKHR::default()
            .color_space(vk::ColorSpaceKHR::SRGB_NONLINEAR)
            .format(vk::Format::R8G8B8A8_SRGB);

        Ok(if formats.contains(&preferred_format) {
            preferred_format
        } else {
            formats[0]
        })
    }

    /// Retuns the best supported present mode.
    #[must_use]
    fn best_present_mode(context: &Context) -> Result<vk::PresentModeKHR> {
        let present_modes = unsafe {
            context
                .surface_instance()
                .get_physical_device_surface_present_modes(
                    context.physical_device(),
                    context.surface(),
                )
                .map_err(|e| Error::VkError(e))?
        };
        let preferred_present_mode = vk::PresentModeKHR::MAILBOX;

        Ok(if present_modes.contains(&preferred_present_mode) {
            preferred_present_mode
        } else {
            vk::PresentModeKHR::FIFO
        })
    }

    /// Gets/calculates the extent.
    #[must_use]
    fn get_extent(window: &Window, capabilities: &vk::SurfaceCapabilitiesKHR) -> vk::Extent2D {
        let is_variable = capabilities.current_extent.width == u32::MAX;
        if is_variable {
            let min_extent = capabilities.min_image_extent;
            let max_extent = capabilities.max_image_extent;
            let width = window.width().clamp(min_extent.width, max_extent.width);
            let height = window.height().clamp(min_extent.height, max_extent.height);

            vk::Extent2D::default().width(width).height(height)
        } else {
            capabilities.current_extent
        }
    }

    /// Gets the number of images.
    #[must_use]
    fn get_num_images(capabilities: &vk::SurfaceCapabilitiesKHR, num_ideal_images: u32) -> u32 {
        let max_num_images = if capabilities.max_image_count == 0 {
            u32::MAX
        } else {
            capabilities.max_image_count
        };
        num_ideal_images.clamp(capabilities.min_image_count, max_num_images)
    }

    /// Destroys the swapchain.
    pub fn destroy(&mut self, device: &Device) {
        let _ = unsafe { device.device_wait_idle() };

        for image in &self.images {
            unsafe { device.destroy_image_view(image.image_view, None) };
        }

        unsafe {
            self.swapchain_device
                .destroy_swapchain(self.swapchain, None)
        };
    }

    /// Returns the swapchain device.
    #[inline]
    #[must_use]
    pub fn device(&self) -> &swapchain::Device {
        &self.swapchain_device
    }

    /// Returns the swapchain.
    #[inline]
    #[must_use]
    pub fn swapchain(&self) -> vk::SwapchainKHR {
        self.swapchain
    }

    /// Returns the swapchain images.
    #[inline]
    #[must_use]
    pub fn images(&self) -> &[SwapchainImage] {
        &self.images
    }

    /// Returnst he swapchain's extent.
    #[inline]
    #[must_use]
    pub fn extent(&self) -> vk::Extent2D {
        self.extent
    }

    /// Returns the swapchain's surface's format.
    #[inline]
    #[must_use]
    pub fn surface_format(&self) -> vk::SurfaceFormatKHR {
        self.surface_format
    }
}
