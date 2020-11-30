use actix_web::{App, get, HttpResponse, HttpServer, post, Responder, web};
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;

mod chord;
mod swim;

pub struct Member {
    counter: i32
}

impl Member {
    pub async fn join(self: &mut Self) {
        self.counter += 1;
    }
}

pub struct Node {}

impl Node {
    pub async fn join(self: &mut Self) {}
}

#[get("/ping")]
async fn ping(data: web::Data<WebAppState>) -> impl Responder {
    // data.handler.incoming(MembershipMessage::Ping).await;
    // {
    //     let mut guard = data.c.lock().unwrap();
    //     *guard += 1;
    // }
    let (tx, rx) = oneshot::channel::<i32>();
    data.tx_member.clone().send(ResponseChannel { msg: MembershipMessage::Join, ch: tx }).await;
    // println!("got: {:?}", rx.await);
    // {
    //     println!("mutex: {}", data.c.lock().unwrap());
    // }
    HttpResponse::Ok().body("Hello world!")
}

#[get("/join")]
async fn ack(data: web::Data<WebAppState>) -> impl Responder {
    // data.handler.incoming(MembershipMessage::Join).await;
    // let (tx, rx) = oneshot::channel();
    // data.tx_member.clone().send(MembershipMessage::Join).await;
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String, _data: web::Data<WebAppState>) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[derive(Debug)]
pub enum MembershipMessage {
    Join,
    Joined,
    Ping,
    RequestPing,
    Ack,
}


pub struct Handler {
    node: Node,
    member: Member,
}

// impl Handler {
//     pub(crate) async fn incoming(&self, message: MembershipMessage) {
//         match message {
//             MembershipMessage::Ping => {
//                 println!("Ping incoming");
//
//             }
//             MembershipMessage::Join => {
//                 println!("Join incoming");
//                 self.member.join();
//             }
//             _ => println!("Somethign else"),
//         }
//     }
// }

struct ResponseChannel {
    ch: tokio::sync::oneshot::Sender<i32>,
    msg: MembershipMessage,
}

struct WebAppState {
    // handler: Handler,
    tx_member: Sender<ResponseChannel>,
    c: Arc<Mutex<i32>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let (mut tx_member, mut rx_member) = tokio::sync::mpsc::channel(4);
    let mut member = Member { counter: 0 };

    let web_app_state = web::Data::new(WebAppState { tx_member, c: Arc::new(Mutex::new(0)) });

    tokio::task::spawn(async move {
        while let Some(message) = rx_member.recv().await {
            // println!("{:?}", &message.msg);
            match &message.msg {
                MembershipMessage::Ping => {
                    println!("Ping incoming");
                }
                MembershipMessage::Join => {
                    member.join().await;
                    message.ch.send(member.counter.clone());
                }
                _ => println!("Somethign else"),
            }
        };
    });

    HttpServer::new(move || {
        App::new()
            .app_data(web_app_state.clone())
            .service(echo)
            .service(ping)
            .service(ack)
            .route("/hey", web::get().to(manual_hello))
    })
        .workers(1)
        .bind("127.0.0.1:8080")?
        .run()
        .await
}