use std::net::TcpListener;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Instant;

use actix::prelude::*;
use actix::SystemRegistry;
use actix_web::dev::Server;
use log::{debug, error, info, LevelFilter, warn};
use reqwest::header::*;
use rstest::{fixture, rstest};
use tokio::sync::oneshot::error::RecvError;
use tokio::task::LocalSet;
use tokio::time::Duration;

use doip::config::{Configuration, Settings};
use doip::net::bootstrap::Net;
use doip::protocols::swim::http_client::Client;
use doip::protocols::swim::swim::SwimActor;
use doip::protocols::swim::messages::ContactAddr;

#[fixture]
pub fn setup() -> () {
    // logger level not working
    let mut logger = log4rs::config::load_config_file("log4rs.yml", Default::default()).unwrap();
    logger.root_mut().set_level(LevelFilter::Debug);
    let _ = log4rs::init_config(logger).unwrap();
}

async fn bootstrap_node(settings: Settings) -> Option<Server> {
    let server = Net::boostrap(settings).await.unwrap();
    let srv = server.clone();
    let (tx, rx) = tokio::sync::oneshot::channel();
    let _ = tokio::spawn(async move {
        if let Err(_) = tx.send(Some(server)) {
            error!("the receiver dropped");
        }
        srv.await.unwrap();
    });

    rx.await.unwrap()
}

#[rstest]
#[actix_rt::test]
async fn run_one_node(_setup: ()) {
    assert_eq!(bootstrap_node(Configuration::new().finish().unwrap()).await.is_some(), true);
}

#[rstest]
#[actix_rt::test]
async fn shutdown_srv(_setup: ()) {
    let port = 9000;
    let srv = bootstrap_node(Configuration::new().port(port).finish().unwrap()).await.unwrap();

    assert_eq!(TcpListener::bind(format!("0.0.0.0:{}", port)).err().is_some(), true);
    srv.stop(true).await;
    assert_eq!(TcpListener::bind(format!("0.0.0.0:{}", port)).err().is_some(), false);
}

#[rstest]
#[actix_rt::test]
async fn same_node_multiple_join(_setup: ()) {
    bootstrap_node(Configuration::new().port(9000).finish().unwrap()).await.unwrap();
    let srv_9001 = bootstrap_node(Configuration::new().port(9001).finish().unwrap()).await.unwrap();
    // TODO
}

#[rstest]
#[actix_rt::test]
async fn join(_setup: ()) {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    let http = reqwest::Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(20))
        .build().unwrap();
    bootstrap_node(Configuration::new().port(9000).finish().unwrap()).await.unwrap();

    for port in 9001..9011 {
        bootstrap_node(Configuration::new().port(port).finish().unwrap()).await.unwrap();
    }

    tokio::time::delay_for(Duration::from_secs(1)).await;
    let members = http.post("http://0.0.0.0:9006/membership").send().await.unwrap()
        .json::<Vec<ContactAddr>>().await.unwrap();
    info!("Found: {:?}", members);
    tokio::time::delay_for(Duration::from_secs(10)).await;
}

#[rstest]
// #[actix_rt::test]
#[tokio::test]
// #[test]
async fn check(_setup: ()) {
    // setup();
    // System::builder().name("first").build().run().unwrap();
    // error!("first f");
    // System::builder().name("second").build().run().unwrap();
    // error!("second f");
    // System::builder().run()
    // let rt = tokio::runtime::Builder::new()
    //     .enable_all()
    //     .build()
    //     .unwrap();
    // .block_on(async {
    //     println!("Hello world");
    // });
    // bootstrap_main_node(Configuration::new().port(9000).finish().unwrap())
    //     .await
    //     .unwrap();

    // std::thread::sleep(Duration::from_secs(2));

    // let handle = tokio::spawn(async {
    let set = LocalSet::new();
    let x = set.run_until(async move {
        let n = LocalSet::new();
        error!("before");

        let tokio2 = System::run_in_tokio("test", &n);
        error!("after");
        let system1 = System::current();

        bootstrap_node(Configuration::new().port(9000).finish().unwrap())
            .await
            .unwrap();

        tokio2.await;
    });
    let set1 = LocalSet::new();
    let (tx, rx) = tokio::sync::oneshot::channel();

    let y = set.run_until(async move {
        let n = LocalSet::new();
        error!("before");
        let tokio1 = System::run_in_tokio("test", &n);
        error!("after");
        let system = System::current();
        tx.send(system);
        tokio::time::delay_for(Duration::from_secs(2)).await;
        bootstrap_node(Configuration::new().port(9001).finish().unwrap())
            .await
            .unwrap();
        error!("waiting before this");
        tokio1.await.unwrap();
        error!("waiting before this1");
    });

    tokio::spawn(async {
        tokio::time::delay_for(Duration::from_secs(10)).await;
        error!("should shutdown now");
        System::current().arbiter().stop();
        System::current().stop();
        System::current().stop_with_code(1);
        tokio::time::delay_for(Duration::from_secs(1)).await;
        System::current().stop();
        System::current().stop_with_code(1);
        assert_eq!(1, 0);
        // tokio::runtime::Runtime::shutdown_timeout()
        std::process::exit(0);
    });

    // let result = rx.await.unwrap();
    // result.stop();

    tokio::join!(x, y);
    // tokio::join!(x, y);
    // });

    // tokio::spawn(async {
    // let set = LocalSet::new();
    // System::run_in_tokio("test", &set).await.unwrap();
    // error!("after");
    // bootstrap_main_node(Configuration::new().port(9000).finish().unwrap())
    //     .await
    //     .unwrap();
    // });


    // for port in 9000..9011 {
    //     System::run(move || {
    //         let _ = tokio::task::spawn(async move {
    //             std::thread::sleep(Duration::from_secs(2));
    //             let settings = Configuration::new().port(port).finish().unwrap();
    //             let net = Net::boostrap(settings).await.unwrap();
    //
    //             let _ = tokio::spawn(net.srv);
    //         });
    //     }).unwrap();
    // }
}