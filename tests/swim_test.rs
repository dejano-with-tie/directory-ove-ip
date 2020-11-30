use std::net::TcpListener;
use std::time::Instant;

use actix::prelude::*;
use actix::SystemRegistry;
use log::{debug, error, info, LevelFilter, warn};
use rstest::{fixture, rstest};
use tokio::sync::oneshot::error::RecvError;
use tokio::task::LocalSet;
use tokio::time::Duration;

use doip::config::{Configuration, Settings};
use doip::net::bootstrap::Net;
use doip::protocols::swim::swim::SwimActor;

#[fixture]
pub fn setup() -> () {
    // logger level not working
    let mut logger = log4rs::config::load_config_file("log4rs.yml", Default::default()).unwrap();
    logger.root_mut().set_level(LevelFilter::Debug);
    let _ = log4rs::init_config(logger).unwrap();
}

async fn bootstrap_node(settings: Settings) -> Result<bool, RecvError> {
    let server = Net::boostrap(settings).await.unwrap();
    let (tx, rx) = tokio::sync::oneshot::channel();
    let _ = tokio::spawn(async move {
        let (_s, _c) = tokio::join!(server, async move {
             if let Err(_) = tx.send(true) {
                error!("the receiver dropped");
            }
        });
    });

    rx.await
}

#[rstest]
#[actix_rt::test]
async fn run_main_node(_setup: ()) {
    assert_eq!(bootstrap_node(Configuration::new().finish().unwrap()).await.unwrap(), true);
}


// #[tokio::test]
async fn test_join() {
    setup();
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let local = tokio::task::LocalSet::new();
    local.block_on(&mut rt, async move {
        tokio::task::spawn_local(async move {
            let local = tokio::task::LocalSet::new();
            let sys = actix_rt::System::run_in_tokio("s", &local);
            // define your actix-web app
            // define your actix-web server
            sys.await;
        });
        // This still allows use of tokio::spawn
    });

    // runner.block_on(async move {
    //     tokio::time::delay_for(time).await;
    // });
    // assert!(
    //     instant.elapsed() >= time,
    //     "Block on should poll awaited future to completion"
    // );
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