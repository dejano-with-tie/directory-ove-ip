use log::debug;

#[derive(serde::Deserialize)]
pub struct Settings {
    pub app_port: u16,
    pub net: Net,
}

#[derive(serde::Deserialize)]
pub struct Net {
    pub known_node: String
}

pub struct Configuration {
    port: Option<u16>
}

impl Configuration {
    pub fn new() -> Self {
        Self { port: None }
    }

    pub fn port(&mut self, port: u16) -> &mut Self {
        self.port = Some(port);
        self
    }
}

impl Configuration {
    // NOTE: Make sure this is called only once with std::sync::once maybe?
    pub fn finish(&self) -> Result<Settings, config::ConfigError> {
        let mut s = config::Config::default();

        s.merge(config::File::with_name("config"))?
            .merge(config::Environment::with_prefix("app"))?;

        if let Some(port) = self.port {
            s.set("app_port", port as i64).unwrap();
        }

        debug!("app_port: {:?}", s.get::<String>("app_port"));
        s.try_into()
    }
}
