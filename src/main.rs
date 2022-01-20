use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
struct SonicbotConfig {
    username: String,
    server_name: String,
    host: String,
    password: String,
    initial_rooms: Vec<String>,
    prefix: String,
    owner: String,
}

fn main() {
    let sonicbot_config: SonicbotConfig = serde_yaml::from_str(&std::fs::read_to_string("config.yaml").unwrap()).unwrap();
    //println!("{:#?}", sonicbot_config);
    let mut inst = sonicbot_matrix::SonicBot::new(sonicbot_config.host, sonicbot_config.username, sonicbot_config.server_name, true, sonicbot_config.prefix, sonicbot_config.owner);
    inst.start(sonicbot_config.password, sonicbot_config.initial_rooms);
}
