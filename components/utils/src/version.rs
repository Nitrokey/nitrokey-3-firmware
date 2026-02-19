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

    pub const fn from_str(s: &str) -> Self {
        // strip pre-release or build components
        let version = split_off(s, b'-');
        let version = split_off(version, b'+');
        // extract major, minor, patch version
        let (major, rest) = split_str(version, b'.').unwrap();
        let (minor, patch) = split_str(rest, b'.').unwrap();
        // parse version numbers
        let major = parse_simple_u8(major);
        let minor = parse_simple_u8(minor);
        let patch = parse_simple_u8(patch);
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

const fn split_str(s: &str, c: u8) -> Option<(&str, &str)> {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == c {
            let Some((first, last)) = s.split_at_checked(i) else {
                return None;
            };
            let Some((_mid, last)) = last.split_at_checked(1) else {
                return None;
            };
            return Some((first, last));
        }
        i += 1;
    }
    None
}

const fn split_off(s: &str, c: u8) -> &str {
    if let Some((start, _)) = split_str(s, c) {
        start
    } else {
        s
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

    #[test]
    fn test_split_str() {
        assert_eq!(split_str("0.1", b'.'), Some(("0", "1")));
        assert_eq!(split_str("0.", b'.'), Some(("0", "")));
        assert_eq!(split_str("0.1.2", b'.'), Some(("0", "1.2")));
        assert_eq!(split_str("012", b'.'), None);
        assert_eq!(split_str("", b'.'), None);
    }

    #[test]
    fn test_version_from_str() {
        let version = Version {
            major: 1,
            minor: 2,
            patch: 3,
        };
        assert_eq!(Version::from_str("1.2.3"), version);
        assert_eq!(Version::from_str("1.2.3-test.1"), version);
        assert_eq!(Version::from_str("1.2.3-test.1+32"), version);
        assert_eq!(Version::from_str("1.2.3+32"), version);
    }

    #[test]
    fn test_version_from_str_cargo() {
        Version::from_str(env!("CARGO_PKG_VERSION"));
    }
}
