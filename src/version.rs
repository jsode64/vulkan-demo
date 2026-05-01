/// A version consisting of variant, major, minor, and patch.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl Version {
    /// Creates a version with the given major, minor, and patch.
    #[inline]
    #[must_use]
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

impl ToString for Version {
    fn to_string(&self) -> String {
        format!("v{}.{}.{}", self.major, self.minor, self.patch)
    }
}
