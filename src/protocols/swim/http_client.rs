use derive_more::Display;
use log::debug;
use reqwest::header::*;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use tokio::time::Duration;

use crate::config::Settings;
use crate::protocols::Error;
use crate::protocols::swim::messages::{Address, ContactAddr};
use crate::protocols::swim::swim::Node;

#[derive(Serialize, Deserialize, Debug, Display)]
pub struct JoinRequest {
    pub address: ContactAddr
}

#[derive(Serialize, Deserialize, Debug, Display)]
#[display(fmt = "{:?}", members)]
pub struct JoinResponse {
    #[serde(flatten)]
    pub members: Vec<ContactAddr>
}

pub struct Client {
    http: reqwest::Client
}

struct Handler<F>
    where
        F: FnMut(Node) -> () {
    method: Method,
    handler: F,
}

pub enum Method {
    Join
}

impl Client {
    // pub fn handle<T>(&self, node: &mut Node, method: &Method) -> Result<T, Error>
    //     where T: Sized {
    //     match method {
    //         Method::Join => {
    //             Ok(())
    //         }
    //     }
    // }

    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        let http = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(20))
            .build().unwrap();
        Self {
            http
        }
    }

    pub async fn join(
        &self,
        payload: &JoinRequest,
        destination: &ContactAddr,
    ) -> Result<ConsumableResponse<JoinResponse>, crate::protocols::Error> {
        debug!("[HTTP Request] Joining network over known node {}", destination);
        let r = self.http.post(&format!("http://{}/join", destination))
            .header("Content-Type", "Application/json")
            .json(payload)
            .send()
            .await?
            .text()
            .await?;
        debug!("[HTTP Response] {:?}", r);
        Ok(ConsumableResponse { r: JoinResponse { members: vec![] } })
        // Ok(r)
    }
}

pub struct ConsumableResponse<T> {
    pub r: T
}


impl ConsumableResponse<JoinResponse> {
    pub fn consume(&mut self, node: &mut Node) -> Node {
        {
            let mut members = node.members.lock().unwrap();
            members.append(&mut self.r.members);
            debug!("Network members: {:?}", *members);
        }
        std::mem::take(node)
    }
}