use crate::protocols::swim::messages::ContactAddr;

pub struct NatDiscovery {}

impl NatDiscovery {
    pub fn find_addr(&self, is_local: &bool, port: &u16) -> ContactAddr {
        if *is_local {
            return ContactAddr(format!("0.0.0.0:{}", port));
        }

        unimplemented!()
    }
}