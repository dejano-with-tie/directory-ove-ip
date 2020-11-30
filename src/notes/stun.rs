use byte::ctx::{BE, Endian};
use byte::TryWrite;
use serde::ser::{SerializeSeq, SerializeStruct};
use serde::Serializer;
use std::convert::TryFrom;
use std::net::SocketAddr;
use std::prelude::*;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpStream, UdpSocket};
use tokio::prelude::io::AsyncReadExt;

// 33, 18, 164, 66
const MAGIC_COOKIE: u32 = 0x2112A442;

#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MessageClass {
    Request = 0,
    Indication = 16,
    SuccessResponse = 256,
    FailureResponse = 272,
}

impl TryFrom<u16> for MessageClass {
    type Error = &'static str;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0b00000000000000 => Ok(MessageClass::Request),
            0b00000000010000 => Ok(MessageClass::Indication),
            0b00000100000000 => Ok(MessageClass::SuccessResponse),
            0b00000100010000 => Ok(MessageClass::FailureResponse),
            _ => Err("Unknown message class given")
        }
    }
}

#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MessageMethod {
    Binding = 1
}

#[derive(Debug)]
pub struct Header {
    class: MessageClass,
    method: MessageMethod,
    transaction_id: [u8; 12],
}

impl Header {
    pub fn new(method: MessageMethod) -> Self {
        Header {
            class: MessageClass::Request,
            method,
            transaction_id: rand::random(),
        }
    }
}


///
///        0                   1                   2                   3
///        0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
///       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///       |0 0|     STUN Message Type     |         Message Length        |
///       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///       |                         Magic Cookie                          |
///       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///       |                                                               |
///       |                     Transaction ID (96 bits)                  |
///       |                                                               |
///       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///
///                   Figure 2: Format of STUN Message Header
///
impl<'a> TryWrite<Endian> for Header {
    fn try_write(self, bytes: &mut [u8], endian: Endian) -> byte::Result<usize> {
        use byte::*;

        let offset = &mut 0;

        bytes.write_with::<u16>(offset, (self.class as u16) | (self.method as u16), endian)?;
        bytes.write_with::<u16>(offset, 0u16 as u16, endian)?;
        bytes.write_with::<u32>(offset, MAGIC_COOKIE, endian)?;

        self.transaction_id.iter().for_each(|f| {
            bytes.write_with::<u8>(offset, f.to_owned(), endian).unwrap();
        });
        Ok(*offset)
    }
}

pub struct Message {
    header: Header
}

impl Message {
    pub fn new(method: MessageMethod) -> Self {
        Message {
            header: Header::new(method)
        }
    }
}

impl<'a> TryWrite<Endian> for Message {
    fn try_write(self, bytes: &mut [u8], ctx: Endian) -> byte::Result<usize> {
        use byte::*;

        let offset = &mut 0;

        bytes.write_with(offset, self.header, ctx);
        Ok(*offset)
    }
}

pub struct Stun {
    addr: SocketAddr
}

impl Stun {
    pub fn new(addr: SocketAddr) -> Self {
        Stun {
            addr
        }
    }
    pub async fn send(&self, method: MessageMethod) {
        use byte::*;
        println!("connecting {:?}", &self.addr);
        let mut sock = UdpSocket::bind("0.0.0.0:9991").await.unwrap();
        sock.connect(self.addr).await.unwrap();

        println!("connected");

        let message = Message::new(method);
        let mut write = [0u8; 20];
        write.write_with(&mut 0, message, BE).unwrap();
        let len = sock.send(&write).await.unwrap();
        println!("{:?} bytes sent", len);


        let mut buf = [0; 1024];
        let len = sock.recv(&mut buf).await.unwrap();
        println!("{:?} bytes received from {:?}", len, self.addr);
        println!("{:?}", buf);
    }
}

#[cfg(test)]
mod tests {
    use byte::*;

    use super::*;

    #[test]
    fn serialize_bind_req() {
        let message = Message::new(MessageMethod::Binding);


        let mut write = [0u8; 8 + 12];
        write.write_with(&mut 0, message, BE).unwrap();
        println!("{:?}", write);
        assert_eq!(write.len(), 8 + 12);
        assert_ne!(&write[8..], [0; 12]);
        // first two for STUN Message Type (MessageClass | MessageMethod)
        // next two bytes are message len -> 0x00, 0x00
        // cookie -> 0x21, 0x12, 0xA4, 0x42
        assert_eq!(&write[..8], [0x00, 0x01, 0x00, 0x00, 0x21, 0x12, 0xA4, 0x42]);
    }

    #[test]
    fn send_msg() {
        use std::net::ToSocketAddrs;
        let server_address = "stun4.l.google.com:19302".to_socket_addrs().unwrap().next()
            .unwrap();
        let stun = Stun::new(server_address);
        stun.send(MessageMethod::Binding).await;
    }
}