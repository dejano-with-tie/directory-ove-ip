use std::borrow::BorrowMut;
use std::convert::TryInto;
use std::error::Error;
use std::fmt;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::{Mutex, RwLock};

use actix::prelude::*;
use log::debug;
use rand::Rng;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use tokio::time::Duration;

pub use crate::protocols::*;
use crate::protocols;
use crate::protocols::swim::http_client;

#[derive(Default)]
pub struct Node {
    pub addr: ContactAddr,
    pub members: RwLock<Vec<ContactAddr>>,
}

#[derive(Default)]
pub struct SwimActor {
    pub node: Node,
}

impl Actor for SwimActor {
    type Context = Context<Self>;
}

impl Supervised for SwimActor {
    fn restarting(&mut self, ctx: &mut Self::Context) {
        println!("For some reason SwimActor has been restarted");
    }
}

impl SystemService for SwimActor {}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tmp {
    pub addr: String
}

// impl Handler<SendJoinRequest> for SwimActor {
//     type Result = ResponseActFuture<Self, Result<Vec<String>, ProtocolError>>;
//
//     fn handle(&mut self, msg: SendJoinRequest, ctx: &mut Context<Self>) -> Self::Result {
//         let addr = format!("http://{}/join", msg.known_addr);
//         let payload = Tmp { addr: addr.clone() };
//         let request = reqwest::Client::default().post(&addr)
//             .header("Content-Type", "Application/json")
//             .json(&payload).send();
//         Box::pin(async {
//             request.await?.json::<Vec<String>>().await
//         }.into_actor(self)
//             .map(|result, _act, _ctx| {
//                 println!("{:?}", result);
//                 match result {
//                     Ok(v) => {
//                         Ok(v)
//                     }
//                     Err(e) => {
//                         _ctx.stop();
//                         Err(ProtocolError::Http(e))
//                     }
//                 }
//             })
//         )
//     }
// }

impl Handler<Join> for SwimActor {
    type Result = MessageResult<Join>;

    fn handle(&mut self, msg: Join, ctx: &mut Context<SwimActor>) -> Self::Result {
        // debug!("{:?}: Received join request from {:?}", self.node.contact_addr, msg);
        //
        // let exist: bool = self.node.members.iter()
        //     .filter(|member| **member == msg.value)
        //     .count() > 0;
        //
        // if exist {
        //     return MessageResult(Err(protocols::Error::Swim("Already joined".into())));
        // }
        //
        // debug!("{:?}: Node [{:?}] successfully joined", self.node.contact_addr, msg);
        // self.node.members.push(msg.value);
        // // let to_gossip = Gossip::pick(&4, &self.node.members);
        // // tokio::spawn(async move {
        // //     Gossip::send(msg, &to_gossip).await;
        // // });

        MessageResult(Ok(vec![]))

        // let f = async {
        //     let resp = http_call.await;
        //     vec![resp.to_string()]
        // };
        // Box::pin(f.into_actor(self))
    }
}