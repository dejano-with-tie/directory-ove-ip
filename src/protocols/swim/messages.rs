use std::ops::Deref;

use actix::prelude::*;
use derive_more::Display;
use serde::{Deserialize, Serialize};

use crate::protocols::*;

pub struct Port(u16);

// TODO: constructor, deref mby, FromStr
#[derive(Serialize, Deserialize, Display, Debug, Default, Eq, PartialEq)]
pub struct ContactAddr(pub String);

#[derive(Message)]
#[rtype(result = "Result<Vec<ContactAddr>, Error>")]
#[derive(Serialize, Deserialize, Debug)]
pub struct Address {
    pub value: ContactAddr
}

pub type Join = Address;

impl Address {
    pub fn new(value: ContactAddr) -> Self {
        Self {
            value
        }
    }
}

// #[derive(Message)]
// #[rtype(result = "Result<Vec<ContactAddr>, Error>")]
// pub struct Join {
//     address: ContactAddr,
// }
//
// impl Join {
//     pub fn new(address: ContactAddr, known_addr: ContactAddr) -> Self {
//         Self {
//             address,
//             known_addr,
//         }
//     }
// }


#[derive(Message)]
#[rtype(result = "Result<String, Error>")]
pub struct JoinRequest {
    pub contact_addr: String
}