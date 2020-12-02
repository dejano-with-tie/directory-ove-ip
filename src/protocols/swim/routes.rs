use std::ops::Deref;

use actix::SystemService;
use actix_web::{post, Responder};
use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use actix_web::web::{Data, Json};
use log::{debug, info};
use rand::Rng;

use crate::net::bootstrap::Net;
use crate::protocols::swim::http_client;
use crate::protocols::swim::swim::*;

fn is_existing_member(members: &Vec<ContactAddr>, addr: &ContactAddr) -> bool {
    members
        .iter()
        .find(|&member| member == addr)
        .is_some()
}

fn remove<'a, 'b>(
    members: &'a Vec<ContactAddr>,
    to_remove: &'b Vec<&ContactAddr>,
) -> Vec<&'a ContactAddr> {
    members.iter()
        .filter(|&addr| to_remove.iter().find(|&&another| another == addr).is_none())
        .collect()
}

#[post("/join")]
pub async fn join(
    net: Data<Net>,
    req: Json<http_client::JoinRequest>,
) -> impl Responder {
    debug!("{:?}: Received join request from {:?}", net.me.addr, req);
    let mut req = req;

    let to_join = std::mem::take(&mut req.address);
    {
        let mut members = net.me.members.read().unwrap();
        if is_existing_member(members.as_ref(), &to_join) {
            //TODO:  not an error actually, should respond with members list
            return HttpResponse::build(StatusCode::BAD_REQUEST).finish();
        }
    }

    {
        let mut members = net.me.members.write().unwrap();
        info!("New node [{:?}] joined network", &to_join.0);
        members.push(ContactAddr(to_join.0.clone()));
    }

    {
        let mut members = net.me.members.read().unwrap();
        debug!("[{:?}] Network members: {:?}", net.me.addr, *members);
        let members_without_me = remove(&members, &vec![&to_join, &net.me.addr]);

        let to_pick = 3;
        let members_without_me: Vec<&ContactAddr> = if members_without_me.len() > to_pick {
            let mut picked = vec![];
            for _i in 0..to_pick {
                let mut rng = rand::thread_rng();
                picked.push(members.get(rng.gen_range(0, members_without_me.len())).unwrap());
            }
            picked
        } else {
            members_without_me
        };

        if !members_without_me.is_empty() {
            let to_gossip: Vec<ContactAddr> = members_without_me.iter().map(|m| {
                ContactAddr((*m).0.clone())
            }).collect();
            // let to_join = ContactAddr(to_join.0.clone());
            tokio::spawn(async move {
                Gossip::send(String::from(format!("joined {}", &to_join.0)), &to_gossip).await;
            });
        }

        HttpResponse::Ok().json(&*net.me.members.read().unwrap())
    }
}

#[post("/membership")]
pub async fn membership(net: Data<Net>) -> impl Responder {
    HttpResponse::Ok().json(&*net.me.members.read().unwrap())
}