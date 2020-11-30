use doip::config::Configuration;
use doip::net::bootstrap::Net;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let _ = log4rs::init_file("log4rs.yml", Default::default()).unwrap();
    let settings = Configuration::new().finish().expect("Failed to read configuration file");
    let server = Net::boostrap(settings).await?;
    server.await
}
