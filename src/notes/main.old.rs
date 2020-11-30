#![feature(async_closure)]

use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, Service, service_fn};
use std::borrow::{Borrow, BorrowMut};
use std::convert::Infallible;
use std::fmt::{Display, Formatter};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::task::Context;
use tokio::macros::support::{Future, Pin, Poll};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::Command::FindSuccessor;

struct DhtService {
    tx: Sender<Command>
}

impl Service<Request<Body>> for DhtService {
    type Response = Response<Body>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output=Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: Request<Body>) -> Self::Future {
        let mut tx = self.tx.clone();
        let x = async move {
            tx.send(Command::FindSuccessor).await;
        };
        tokio::spawn(x);
        // tokio::spawn(async move {
        //     tx.send(Command::FindSuccessor).await;
        // });
        let res = Ok(Response::builder().body(Body::from("Hello world".to_owned())).unwrap());
        println!("hello");
        Box::pin(async { res })
    }
}

struct MakeDht {
    tx: Sender<Command>
}

impl<T> Service<T> for MakeDht {
    type Response = DhtService;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output=Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: T) -> Self::Future {
        let tx = self.tx.clone();
        let fut = async move { Ok(DhtService { tx }) };
        Box::pin(fut)
    }
}

pub struct Node {
    pub ip: String,
    pub id: String,
    pub predecessor: Box<Option<Node>>,
    pub successor: Box<Option<Node>>,
}

impl Node {
    pub fn new(ip: &str) -> Node {
        let predecessor = Box::new(None);
        let successor = Box::new(None);

        Node {
            ip: String::from(ip),
            id: String::from(ip),
            predecessor,
            successor,
        }
    }

    pub fn join(&mut self, node: &Node) {
        self.predecessor = Box::new(None);
        // self.successor = node.find_successor(self.borrow());
    }

    pub fn find_successor(&self, _node: &Node) -> Box<Option<Node>> {
        // TODO: Implement
        Box::new(None)
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{} ({})]", self.ip, self.id)
    }
}

pub enum Command {
    FindSuccessor
}

pub struct Chord {
    rx: Box<Receiver<Command>>
}

impl Chord {
    pub fn new(rx: Receiver<Command>) -> Self {
        Chord { rx: Box::new(rx) }
    }

    pub async fn listen(self: &mut Self) {
        // Start receiving messages
        // let mut rx: &mut Receiver<Command> = self.rx.borrow_mut();
        while let Some(cmd) = self.rx.recv().await {
            match cmd {
                FindSuccessor => {
                    println!("Find successor command");
                    self.find_successor();
                }
            }
        }
    }

    fn find_successor(self: &mut Self) {
        println!("fin successor called chord");
    }
}

async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}

#[tokio::main]
async fn main() {
    let node = Node::new("1");

    let addr = ([127, 0, 0, 1], 3000).into();

    let (mut tx, mut rx) = tokio::sync::mpsc::channel(32);
    // tokio::task::spawn(async move {
    //     // Start receiving messages
    //     while let Some(cmd) = rx.recv().await {
    //         match cmd {
    //             FindSuccessor => {
    //                 println!("Find successor command");
    //             }
    //         }
    //     }
    // });
    let mut chord = Chord::new(rx);
    tokio::spawn(async move {
        chord.listen().await;
    });

    let srv = tokio::task::spawn(async move {
        // let chord = Chord::new(rx);
        let server = Server::bind(&addr).serve(MakeDht { tx });

        // And now add a graceful shutdown signal...
        let graceful = server.with_graceful_shutdown(shutdown_signal());
        // Run this server for... forever!
        if let Err(e) = graceful.await {
            eprintln!("server error: {}", e);
        }
    });

    srv.await.unwrap();

    loop {}
}
