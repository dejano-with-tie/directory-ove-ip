use std::ops::Deref;

use actix::SystemService;
use actix_web::{post, Responder};
use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use log::debug;

use crate::net::bootstrap::Net;
use crate::protocols::swim::swim::*;

#[post("/join")]
pub async fn join(net: actix_web::web::Data<Net>, req:
actix_web::web::Json<crate::protocols::swim::http_client::JoinRequest>) -> impl
Responder {
    debug!("{:?}: Received join request from {:?}", net.me.addr, req);

    let mut members = net.me.members.lock().unwrap();
    if members
        .iter()
        .find(|member| **member == req.address)
        .is_some() {
        return HttpResponse::build(StatusCode::BAD_REQUEST).finish();
    }
    debug!("{:?}: Node [{:?}] successfully joined", net.me.addr, &req.address.0);
    members.push(ContactAddr(req.address.0.clone()));
    HttpResponse::Ok().json(&*members)


    // let to_gossip = Gossip::pick(&4, &self.node.members);
    // tokio::spawn(async move {
    //     Gossip::send(msg, &to_gossip).await;
    // });

    // let f = async {
    //     let resp = http_call.await;
    //     vec![resp.to_string()]
    // };
    // Box::pin(f.into_actor(self))
    // match SwimActor::from_registry()
    //     .send(Join::new(addr.into_inner().value))
    //     .await.unwrap() {
    //     Err(_e) => HttpResponse::build(StatusCode::BAD_REQUEST).finish(),
    //     Ok(members) => HttpResponse::Ok().json(members)
}