//! Version Information & Support

#[derive(Debug)]
pub struct Version {
    major: u8,
    minor: u8,
    patch: u8,
}

impl Version {
    pub fn major() -> u8 { major }
    pub fn minor() -> u8 { minor }
    pub fn patch() -> u8 { patch }
}

impl Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.{})", self.major, self.minor, self.patch)
    }
}

/// Create a Macro from Cargo.
macro_rules! version {
    () => {
        Version {
            major: env!("CARGO_PKG_VERSION_MAJOR"),
            minor: env!("CARGO_PKG_VERSION_MINOR"),
            patch: env!("CARGO_PKG_VERSION_PATCH"),
        }
    }
}

/// Hivemind Version Constant
pub const VERSION: Version = version!();