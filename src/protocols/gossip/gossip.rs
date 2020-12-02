use rand::prelude::ThreadRng;
use rand::Rng;
use tokio::time::Duration;

use crate::protocols::swim::swim::*;
use crate::protocols::swim::http_client::Client;

#[derive(Debug)]
pub enum GossipMessage {
    // TODO: Need generic way for second param (payload)
    Join(&'static str, String)
}

pub struct Gossip {
    pub select_strategy: Box<dyn SelectStrategy>,
    pub how_many: u16,
    pub http: Client
}

pub trait SelectStrategy {
    fn select(&self, nodes: &Vec<ContactAddr>, how_many: u16) -> Vec<&ContactAddr>;
}

pub struct RandomSelectStrategy(pub ThreadRng);

impl SelectStrategy for RandomSelectStrategy {
    fn select(&mut self, nodes: &Vec<ContactAddr>, how_many: u16) -> Vec<&ContactAddr> {
        (0..how_many)
            .map(|_i| nodes.get(self.0.gen_range(0, nodes.len())).unwrap())
            .collect()
    }
}

impl Gossip {
    /// nodes to which gossip message will be sent
    pub async fn gossip(&self, message: &GossipMessage, nodes: &Vec<ContactAddr>) -> () {
        self.select_strategy.select(nodes, self.how_many)
            .iter()
            .for_each(|&member| self.send(message, member).await);
    }

    pub async fn send(&self, message: &GossipMessage, member: &ContactAddr) {
        tokio::time::delay_for(Duration::from_secs(4)).await;
        members.iter().for_each(|m| {
            println!("gossip -> [{:?}] about: {}", &message, (*m).0);
        })
    }
}
