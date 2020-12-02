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
    pub http: reqwest::Client
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

    pub async fn join<'a>(
        &self,
        payload: &JoinRequest,
        destination: &ContactAddr,
    ) -> Result<ConsumableResponse<'a, Vec<ContactAddr>>, crate::protocols::Error> {
        debug!("[HTTP Request] Joining network over known node {}", destination);
        let endpoint = "join";
        let r = self.http.post(&format!("http://{}/{}", destination, endpoint))
            .header("Content-Type", "Application/json")
            .json(payload)
            .send()
            .await?
            .json::<Vec<ContactAddr>>()
            .await?;
        debug!("[HTTP Response] {:?}", r);
        Ok(ConsumableResponse { body: r, endpoint })
    }
}

pub struct ConsumableResponse<'a, T> {
    pub body: T,
    pub endpoint: &'a str,
}

impl<'a> ConsumableResponse<'a, Vec<ContactAddr>> {
    pub fn consume(&mut self, node: &mut Node) -> Node {
        match self.endpoint {
            "join" => {
                {
                    let mut members = node.members.write().unwrap();
                    members.append(&mut self.body);
                    debug!("Network members: {:?}", *members);
                }
                std::mem::take(node)
            }
            _ => {
                panic!("Unknown endpoint");
            }
        }
    }
}