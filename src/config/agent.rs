use config::Config;

#[derive(Debug)]
#[derive(RustcDecodable, RustcEncodable)]
pub struct AgentConf {
    pub listen_port: i32,
}

impl Config for AgentConf {
    type ConfigFile = AgentConf;
}