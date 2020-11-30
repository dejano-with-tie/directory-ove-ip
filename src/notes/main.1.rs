#![allow(dead_code)]

use byte::{BytesExt, TryRead};
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use rand::prelude::*;
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::convert::{Infallible, TryFrom};
use std::fmt::{Display, Formatter};
use std::io::Write;
use std::net::{SocketAddr, SocketAddrV4};
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use stun::{Client, IpVersion, Message};
use tokio::io::Error;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs, UdpSocket};
use tokio::prelude::*;
use tokio::sync::mpsc::{Receiver, Sender};

// mod stun;

pub struct NodeDesc {
    pub addr: SocketAddrV4,
    pub key: Key,
    pub n: Arc<Mutex<u32>>,
}

struct HttpServer;

struct HttpClient;

struct Node {
    http_server: Box<Receiver<String>>,
    http_client: HttpClient,
    nat_discovery: NatDiscovery,
    desc: NodeDesc,
}

impl Display for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}[{}]", self.desc.addr, self.desc.key)
    }
}

impl Node {
    pub fn new(http_server: Receiver<String>, http_client: HttpClient, nat_discovery: NatDiscovery,
               desc:
               NodeDesc) -> Self {
        let mut node = Node {
            http_server: Box::new(http_server),
            http_client,
            nat_discovery,
            desc,
        };

        node
    }

    pub async fn handle(self: &Self) {
        let mut n = self.desc.n.lock().unwrap();
        *n += 1;
        println!("{} handle", n);
    }
    pub async fn listen(self: &mut Self) {
        // Start receiving messages
        // let mut rx: &mut Receiver<Command> = self.rx.borrow_mut();
        while let Some(cmd) = self.http_server.recv().await {
            // println!("{}", self.desc.n.borrow());
            println!("Command: {}", cmd);
        }
        println!("after listen");
    }
}

struct NatDiscovery {}

impl NatDiscovery {
    pub async fn find_ip(&self, is_local: &bool, port: &u16) -> SocketAddrV4 {
        if *is_local {
            return format!("0.0.0.0:{}", port).parse::<SocketAddrV4>().unwrap();
        }

        unimplemented!()
    }
}

struct NodeBuilder {
    local: bool,
    port: u16,
}

impl NodeBuilder {
    pub fn new() -> Self {
        let mut rng = thread_rng();

        NodeBuilder { local: false, port: rng.gen_range(3490, 4000) }
    }
    pub fn local(&mut self) -> &mut Self {
        self.local = true;
        self
    }

    pub fn port(&mut self, port: &u16) -> &mut Self {
        self.port = port.clone();
        self
    }

    pub async fn bootstrap(&self) {
        let nat = NatDiscovery {};

        let mut ip = nat.find_ip(&self.local, &self.port).await;
        ip.set_port(self.port);
        let key = Key::new(&ip);
        let desc = NodeDesc {
            addr: ip,
            key,
            n: Arc::new(Mutex::new(0)),
        };

        let server = HttpServer {};
        let client = HttpClient {};

        let (mut tx, mut rx) = tokio::sync::mpsc::channel(32);
        let srv_addr = SocketAddr::V4(ip.clone());
        // run server
        // make_service_fn running on each connection
        tokio::spawn(async move {
            let tx = tx.clone();
            let make_svc = make_service_fn(move |_conn| {
                let tx = tx.clone();
                async {
                    // service_fn running on each request
                    Ok::<_, Infallible>(service_fn(move |req| {
                        let tx = tx.clone();
                        async move {
                            NodeBuilder::incoming(req, tx).await
                        }
                    }))
                }
            });

            let server = Server::bind(&srv_addr).serve(make_svc);
            if let Err(e) = server.await {
                eprintln!("server error: {}", e);
            }
        });

        // construct client
        let (mut tx1, mut rx1) = tokio::sync::mpsc::channel(32);
        // let node = Node::new(rx1, client, nat, desc);
        // tokio::spawn(async move {
        //     node.listen().await;
        //     println!("{}", node);
        // });

        // let another = &node;
        let handle = tokio::spawn(async move {
            let node = Node::new(rx1, client, nat, desc);
            // tokio::spawn(async move {
            //     node.listen().await;
            //     println!("{}", node);
            // });

            let another = &node;
            while let Some(cmd) = rx.recv().await {
                another.handle().await;
                println!("Command1: {}", cmd);
            }
            println!("after listen");
        });
        println!("after1");
        handle.await.unwrap();
        println!("after2");

        // println!("{}", node.desc.n.deref().lock().unwrap());
    }

    pub async fn incoming(req: Request<Body>, mut tx: Sender<String>) -> Result<Response<Body>,
        Infallible> {
        tx.send("Command::FindSuccessor".to_owned()).await;
        Ok::<_, Infallible>(Response::<Body>::new("Hello world!".into()))
    }
}

pub struct Key {
    pub id: [u8; RING_SIZE],
}

impl Key {
    pub fn new(addr: &SocketAddrV4) -> Key {
        let digested = sha1::Sha1::from(addr.to_string().as_bytes()).digest().bytes();
        let mut id: [u8; RING_SIZE] = [0; RING_SIZE];
        id.copy_from_slice(&digested[0..RING_SIZE]);
        Key { id }
    }
}

impl Display for Key {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02x?}", self.id)
    }
}

const KNOWN_NODES: [&'static str; 1] = ["0.0.0.0:3400"];
const RING_SIZE: usize = 20;

#[tokio::main]
async fn main() {
    NodeBuilder::new().local().port(&3400).bootstrap().await;

    loop {}
}

//
// async fn test() {
//     // let mut stream = TcpStream::connect("127.0.0.1:3682").await?;
//     // stream.write_all(b"bytes").await?;
//     // Ok(())
//     use tokio::net::ToSocketAddrs;
//
//     let stun = Stun::new("127.0.0.1:3682".parse().unwrap());
//     stun.send(Message::new()).await;
// }