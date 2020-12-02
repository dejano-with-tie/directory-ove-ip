use std::error::Error;
use std::net::TcpListener;
use std::sync::{Arc, Mutex, RwLock};

use actix::{Actor, Addr, Supervisor, SystemRegistry};
use actix_rt::System;
use actix_web::{App, HttpServer};
use actix_web::dev::{Server, Service};
use actix_web::middleware::Logger;
use log::{debug, info};
use reqwest::Client;

use crate::config::Settings;
use crate::http::routes::*;
use crate::protocols::gossip::gossip::Message::Join;
use crate::protocols::nat::discovery::*;
use crate::protocols::swim::http_client;
use crate::protocols::swim::http_client::JoinRequest;
use crate::protocols::swim::messages::{Address, ContactAddr};
use crate::protocols::swim::swim::{Node, SwimActor};
use crate::protocols::gossip::gossip::Gossip;

pub struct Inner {
    pub settings: Settings,
    pub http_client: http_client::Client,
    pub me: Node,
}

pub struct Net {
    pub gossip: Gossip
}

impl Net {
    pub fn new(settings: Settings, http_client: http_client::Client, node: Node) -> Self {
        let net = Net {
            settings,
            http_client,
            me: node,
        };

        net
    }

    pub async fn boostrap(settings: Settings) -> std::io::Result<Server> {
        let nat = NatDiscovery {};

        let contact_addr = nat.find_addr(&true, &settings.app_port);
        let node = Node {
            addr: contact_addr,
            members: RwLock::new(vec![]),
        };

        // create server instance
        let addr = format!("0.0.0.0:{}", &settings.app_port);
        let l = match TcpListener::bind(&addr) {
            Ok(l) => l,
            Err(e) => {
                System::current().stop_with_code(1);
                eprintln!("System error: Failed to bind to {}\n{}", &addr, e);
                std::process::exit(1);
            }
        };
        let http_client = http_client::Client::new();
        let mut net = Net::new(settings, http_client::Client::new(), node);
        match net.introduce().await {
            Err(e) => {
                System::current().stop_with_code(1);
                eprintln!("Application error: Failed to contact known node;\n{}", e);
                std::process::exit(1);
            }
            _ => {}
        };

        Ok(Net::create_srv(l, net)?)
    }

    async fn introduce(&mut self) -> Result<(), crate::protocols::Error> {
        let known_node_addr = self.settings.net.known_node.clone();
        let me = self.me.addr.0.clone();
        if me == known_node_addr {
            debug!("I'm boostrap node, starting network");
            // self.me.members.
            self.me.members.write().unwrap().push(ContactAddr(known_node_addr.clone()));
            return Ok(());
        }

        let payload = JoinRequest { address: ContactAddr(me) };
        self.me = self.http_client
            .join(&payload, &ContactAddr(known_node_addr.clone()))
            .await?
            .consume(&mut self.me);
        Ok(())
        // self.http_client.join(&payload, &ContactAddr(known_node_addr)).await
    }

    pub fn create_srv(listener: TcpListener, net: Net) -> std::io::Result<Server> {
        let addr = listener.local_addr().unwrap();
        let node = actix_web::web::Data::new(net);
        let server = HttpServer::new(move || {
            App::new()
                .app_data(node.clone())
                .wrap(Logger::default())
                .service(health)
                .service(echo)
                .service(crate::protocols::swim::routes::join)
                .service(crate::protocols::swim::routes::membership)
        })
            .listen(listener)?
            .run();
        debug!("Booting http server on {}", addr);
        Ok(server)
    }
}