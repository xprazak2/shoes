use crate::handshake::{method::SocksMethod, version::SocksVersion};

// #[derive(Debug)]
pub struct ClientConnectMsg {
    version: SocksVersion,
    n_methods: u8,
    methods: Vec<SocksMethod>,
}

impl ClientConnectMsg {
    pub fn new(version: SocksVersion, n_methods: u8, methods: Vec<SocksMethod>) -> Self {
        Self {
            version,
            n_methods,
            methods,
        }
    }

    pub fn to_request(&self) -> Vec<u8> {
        let mut req = vec![];
        req.push(self.version.into());
        req.push(self.n_methods);
        for method in self.methods.iter() {
            req.push((*method).into());
        }
        req
    }
}
