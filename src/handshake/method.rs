#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SocksMethod {
    NoAuth,
    NoAcceptableMethod,
}

impl From<u8> for SocksMethod {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::NoAuth,
            _ => Self::NoAcceptableMethod,
        }
    }
}

impl From<SocksMethod> for u8 {
    fn from(method: SocksMethod) -> Self {
        match method {
            SocksMethod::NoAuth => 0x00,
            SocksMethod::NoAcceptableMethod => 0xFF,
        }
    }
}
