use serde::Deserialize;
use std::path::{PathBuf};
use std::io::Write;
#[cfg(target_os = "android")]
use macroquad::prelude::*;

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

#[cfg(target_os = "android")]
#[macroquad::main("sonicbot_matrix")]
async fn main() {
    if !handle_config() {
        info!("[sonicbot-matrix] handle_config is false");
        let mut line_wrapper = linewrapper::LineWrapper::new();
        //info!("[sonicbot-matrix] Past line_wrapper creation");
        linewrapper::lw_println!(line_wrapper, "Could not find config file ({}).  Creating blank config...\nPlease fill out config.yaml in the {} directory.", get_config_path().join("config.yaml").to_string_lossy(), get_config_path().to_string_lossy());
        //info!("[sonicbot-matrix] Past macro");
        loop {
            info!("[sonicbot-matrix] In loop");
            line_wrapper.show_lines();
            next_frame().await;
        }
    }
    //info!("[sonicbot-matrix] handle_config is true");
    let sonicbot_config: SonicbotConfig = serde_yaml::from_str(&std::fs::read_to_string(get_config_path().join("config.yaml")).unwrap()).unwrap();
    //println!("{:#?}", sonicbot_config);
    let inst = sonicbot_matrix::SonicBot::new(sonicbot_config.host, sonicbot_config.username, sonicbot_config.server_name, true, sonicbot_config.prefix, sonicbot_config.owner);
    inst.start(sonicbot_config.password, sonicbot_config.initial_rooms).await;
}

#[cfg(target_os = "android")]
fn get_config_path() -> PathBuf {
    PathBuf::from("/storage/emulated/0/Android/media/rust.sonicbot_matrix")
}

#[cfg(not(target_os = "android"))]
fn get_config_path() -> PathBuf {
    PathBuf::from("./")
}
fn handle_config() -> bool {
    let dist_config = include_str!("../config.yaml.dist");
    if !get_config_path().exists() {
        std::fs::create_dir(get_config_path()).unwrap();
    }
    if !get_config_path().join("config.yaml").exists() {
        let mut f = std::fs::File::create(get_config_path().join("config.yaml")).unwrap();
        f.write_all(dist_config.as_bytes()).unwrap();
        return false;
    }
    true
    
}

#[cfg(not(target_os = "android"))]
fn main() {
    if !handle_config() {
        let _line_wrapper = linewrapper::LineWrapper::new();
        linewrapper::lw_println!(_line_wrapper, "{}", "Could not find config file (config.yaml).  Creating blank config...\nPlease fill out config.yaml in the current directory.");
        //if !get_config_path().exists() {
        //    std::fs::create_dir_all(get_config_path()).unwrap();
        //}
        return;
    }

    let sonicbot_config: SonicbotConfig = serde_yaml::from_str(&std::fs::read_to_string(get_config_path().join("config.yaml")).unwrap()).unwrap();
    //println!("{:#?}", sonicbot_config);
    let mut inst = sonicbot_matrix::SonicBot::new(sonicbot_config.host, sonicbot_config.username, sonicbot_config.server_name, true, sonicbot_config.prefix, sonicbot_config.owner);
    inst.start(sonicbot_config.password, sonicbot_config.initial_rooms);
}

