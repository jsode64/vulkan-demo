use glfw::{Glfw, PWindow};

use crate::{Result, Vec2};

/// A builder for a window.
pub struct WindowBuilder<'a> {
    /// The window's width.
    width: u32,

    /// The window's height.
    height: u32,

    /// The window's title.
    title: &'a str,
}

/// A window.
pub struct Window {
    /// The GLFW instance.
    glfw: Glfw,

    /// The GLFW window handle.
    window: PWindow,

    /// The window's width.
    width: u32,

    /// The window's height.
    height: u32,
}

impl<'a> WindowBuilder<'a> {
    /// Builds the window.
    #[inline]
    #[must_use]
    pub fn build(self) -> Result<Window> {
        Window::new(self)
    }

    /// Sets the window's width.
    #[inline]
    #[must_use]
    pub const fn width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    /// Sets the window's height.
    #[inline]
    #[must_use]
    pub const fn height(mut self, height: u32) -> Self {
        self.height = height;
        self
    }

    /// Sets the window's size (width and height).
    #[inline]
    #[must_use]
    pub const fn size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Sets the window's title.
    #[inline]
    #[must_use]
    pub const fn title(mut self, title: &'a str) -> Self {
        self.title = title;
        self
    }
}

impl Window {
    /// Returns a default window builder.
    ///
    /// By default, the window builder will be 800x450 with no title.
    #[inline]
    #[must_use]
    pub const fn builder<'a>() -> WindowBuilder<'a> {
        WindowBuilder {
            width: 800,
            height: 450,
            title: "",
        }
    }

    /// Creates a window from the builder.
    #[inline]
    #[must_use]
    pub fn new(builder: WindowBuilder) -> Result<Self> {
        let mut glfw = glfw::init_no_callbacks().unwrap();

        glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
        let (window, _) = glfw
            .create_window(
                builder.width,
                builder.height,
                builder.title,
                glfw::WindowMode::Windowed,
            )
            .unwrap();

        let (width, height) = window.get_framebuffer_size();

        Ok(Self {
            glfw,
            window,
            width: width as u32,
            height: height as u32,
        })
    }

    /// Returns the window's width.
    #[inline]
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Returns the window's height.
    #[inline]
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Returns whether the window should close.
    #[inline]
    #[must_use]
    pub fn should_close(&self) -> bool {
        self.window.should_close()
    }

    /// Returns the mose position.
    #[must_use]
    pub fn mouse_pos(&self) -> Vec2<f32> {
        let (x, y) = self.window.get_cursor_pos();
        Vec2::new(
            x as f32 - (self.width as f32 / 2.0),
            y as f32 - (self.height as f32 / 2.0),
        )
    }

    /// Updates the window.
    #[inline]
    pub fn update(&mut self) {
        self.glfw.poll_events();

        let (width, height) = self.window.get_framebuffer_size();
        self.width = width as u32;
        self.height = height as u32;
    }

    /// Returns the GLFW instance.
    #[inline]
    #[must_use]
    pub fn glfw(&self) -> &Glfw {
        &self.glfw
    }

    /// Returns the GLFW window handle.
    #[inline]
    #[must_use]
    pub fn window(&self) -> &PWindow {
        &self.window
    }
}
