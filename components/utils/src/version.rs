#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Version {
    major: u8,
    minor: u8,
    patch: u8,
}

impl Version {
    pub const fn new(major: u8, minor: u8, patch: u8) -> Self {
        if patch >= 64 {
            panic!("patch version must not be larger than 63");
        }
        Self {
            major,
            minor,
            patch,
        }
    }

    pub const fn from_env() -> Self {
        let major = parse_simple_u8(env!("CARGO_PKG_VERSION_MAJOR"));
        let minor = parse_simple_u8(env!("CARGO_PKG_VERSION_MINOR"));
        let patch = parse_simple_u8(env!("CARGO_PKG_VERSION_PATCH"));
        Self::new(major, minor, patch)
    }

    pub const fn encode(&self) -> u32 {
        ((self.major as u32) << 22) | ((self.minor as u32) << 6) | (self.patch as u32)
    }

    pub const fn usb_release(&self) -> u16 {
        u16::from_be_bytes([self.major, self.minor])
    }

    pub const fn major(&self) -> u8 {
        self.major
    }

    pub const fn minor(&self) -> u8 {
        self.minor
    }

    pub const fn patch(&self) -> u8 {
        self.patch
    }
}

const fn parse_simple_u8(s: &str) -> u8 {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        panic!("number may not be empty");
    }
    let mut value = 0;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] < b'0' || bytes[i] > b'9' {
            panic!("number must only contain ASCII digits");
        }
        value *= 10;
        value += bytes[i] - b'0';
        i += 1;
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    quickcheck::quickcheck! {
        fn test_parse_simple_u8(value: u8) -> bool {
            parse_simple_u8(&value.to_string()) == value
        }
    }
}
