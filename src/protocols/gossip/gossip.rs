use rand::Rng;
use tokio::time::Duration;

use crate::protocols::swim::swim::*;

pub trait GossipMessage {
    fn payload(self: &Self) -> String;
    fn id(self: &Self) -> Message;
}

impl GossipMessage for JoinRequest {
    fn payload(&self) -> String {
        self.contact_addr.clone()
    }

    fn id(&self) -> Message {
        Message::Join
    }
}

#[derive(Debug)]
pub enum Message {
    Join
}

pub struct Gossip;

impl Gossip {
    /// nodes to which gossip message will be sent
    pub fn pick(how_many: &u32, members: &Vec<String>) -> Vec<String> {
        let mut picked = vec![];
        for i in 0..*how_many {
            let mut rng = rand::thread_rng();
            picked.push(members.get(rng.gen_range(0, members.len())).unwrap().clone());
        }

        picked
    }

    pub async fn send<T: GossipMessage>(message: T, members: &Vec<String>) {
        tokio::time::delay_for(Duration::from_secs(4)).await;
        println!("gossip about {:?}: {}", message.id(), message.payload());
        tokio::spawn(async move {});
    }
}
