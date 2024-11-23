use std::collections::HashMap;

#[derive(Debug)]
pub struct ServiceData {
    pub name: String,
    pub port: String,
    pub host: String,
    pub auth_only: bool,
}

pub struct ServiceMap {
    services: HashMap<String, ServiceData>,
}

impl ServiceMap {
    pub fn new() -> ServiceMap {
        ServiceMap {
            services: HashMap::new(),
        }
    }

    pub fn add_service(
        &mut self,
        name: &str,
        endpoint: &str,
        default_host: &str,
        host_env: &str,
        default_port: u32,
        port_env: &str,
        auth_only: bool,
    ) {
        let port = std::env::var(port_env).unwrap_or(default_port.to_string());
        let host = std::env::var(host_env).unwrap_or(default_host.to_string());
        self.services.insert(
            endpoint.to_string(),
            ServiceData {
                name: name.to_string(),
                port: port,
                host: host,
                auth_only: auth_only,
            },
        );
    }

    pub fn get_service(&self, endpoint: &str) -> Option<&ServiceData> {
        self.services.get(endpoint)
    }
}
