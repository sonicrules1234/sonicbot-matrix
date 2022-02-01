use uuid::Uuid;
use ruma::{DeviceKeyAlgorithm, RoomId, UserId, UInt, presence::PresenceState, serde::Raw, api::client::r0::sync::sync_events::*};
use std::collections::BTreeMap;
use ureq::{Agent, AgentBuilder, OrAnyStatus};
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};
use std::time::{Instant, Duration};
use linewrapper::LineWrapper;
use std::sync::mpsc::{channel, Sender, Receiver};
#[cfg(target_os = "android")]
use macroquad::prelude::*;

pub mod macros;
pub mod essentials;
pub mod plugins;
mod instruction_generators;
use instruction_generators::RoomTypeData;
const BASE: &str = "/_matrix/client/v3/";
const TIME_TO_IDLE: Duration = Duration::from_secs(3 * 60);
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct StorageData {
    pub auth_token: String,
}

#[allow(dead_code)]
pub struct SonicBotEventModule<'a> {
    pub name: String,
    pub essential: bool,
    pub main: Box<dyn Fn(EventArgs<'a>) -> Vec<crate::Instructions>>,
    help: String,
}
#[derive(Debug, Clone)]
pub struct MessageArgs<'a> {
    pub message_info: MessageInfo,
    pub owner: UserId,
    pub ctrlc_handler: &'a ctrlc_handler::CtrlCHandler,
    pub cleanup_on_ctrlc: bool,
    pub prefix: String,
}

impl<'a> MessageArgs<'a> {
    pub fn new(message_info: MessageInfo, owner: UserId, ctrlc_handler: &'a ctrlc_handler::CtrlCHandler, cleanup_on_ctrlc: bool, prefix: String) -> Self {
        Self {
            message_info: message_info,
            owner: owner,
            ctrlc_handler: ctrlc_handler,
            cleanup_on_ctrlc: cleanup_on_ctrlc,
            prefix: prefix,
        }
    }
}


#[derive(Debug, Clone)]
pub struct EventArgs<'a> {
    pub room_data: crate::instruction_generators::RoomTypeData,
    pub starting: bool,
    pub ctrlc_handler: &'a ctrlc_handler::CtrlCHandler,
    pub cleanup_on_ctrlc: bool,
    pub owner: UserId,
    pub prefix: String,
    pub me: UserId,
    pub tx: Sender<String>,
}

impl<'a> EventArgs<'a> {
    pub fn new(room_data: crate::instruction_generators::RoomTypeData, starting: bool, ctrlc_handler: &'a ctrlc_handler::CtrlCHandler, cleanup_on_ctrlc: bool, owner: UserId, prefix: String, me: UserId, tx: Sender<String>) -> Self {
        Self {
            room_data: room_data,
            starting: starting,
            ctrlc_handler: ctrlc_handler,
            cleanup_on_ctrlc: cleanup_on_ctrlc,
            owner: owner,
            prefix: prefix,
            me: me,
            tx: tx.clone()
        }
    }
}

pub struct SonicBotMessageModule<'a> {
    pub name: String,
    pub essential: bool,
    pub main: Box<dyn Fn(MessageArgs<'a>) -> Vec<crate::Instructions>>,
    pub help: String,
}

pub fn generate_module_names(glob_results: glob::Paths) -> Vec<String> {
    let mut module_names: Vec<String> = Vec::new();
    for r in glob_results {
        let file_name: String = r.unwrap().as_path().file_name().unwrap().to_str().unwrap().to_string();
        let module_name = file_name.split(".").collect::<Vec<&str>>()[0];
        if module_name != "mod" {
            module_names.push(module_name.to_string());
        }
    }
    module_names.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    module_names
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct EventResponse {
    /// The batch token to supply in the `since` param of the next `/sync` request.
    pub next_batch: String,

    /// Updates to rooms.
    #[serde(default, skip_serializing_if = "Rooms::is_empty")]
    pub rooms: Rooms,

    /// Updates to the presence status of other users.
    #[serde(default, skip_serializing_if = "Presence::is_empty")]
    pub presence: Presence,

    /// The global private data created by this user.
    #[serde(default, skip_serializing_if = "GlobalAccountData::is_empty")]
    pub account_data: GlobalAccountData,

    /// Messages sent directly between devices.
    #[serde(default, skip_serializing_if = "ToDevice::is_empty")]
    pub to_device: ToDevice,

    /// Information on E2E device updates.
    ///
    /// Only present on an incremental sync.
    #[serde(default, skip_serializing_if = "DeviceLists::is_empty")]
    pub device_lists: DeviceLists,

    /// For each key algorithm, the number of unclaimed one-time keys
    /// currently held on the server for a device.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub device_one_time_keys_count: BTreeMap<DeviceKeyAlgorithm, UInt>,

    /// For each key algorithm, the number of unclaimed one-time keys
    /// currently held on the server for a device.
    ///
    /// The presence of this field indicates that the server supports
    /// fallback keys.
    #[serde(rename = "org.matrix.msc2732.device_unused_fallback_key_types")]
    pub device_unused_fallback_key_types: Option<Vec<DeviceKeyAlgorithm>>,
}

#[derive(Debug)]
pub struct SonicBot {
    data: StorageData,
    host: String,
    me: UserId,
    agent: Agent,
    joined_rooms: Vec<ruma::RoomId>,
    since: Option<String>,
    last_response_time: Instant,
    starting: bool,
    ctrlc_handler: ctrlc_handler::CtrlCHandler,
    cleanup_on_ctrlc: bool,
    prefix: String,
    owner: UserId,
    line_wrapper: Option<Sender<String>>,
}
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Instructions {
    UpdateLastResponseTime(Instant),
    AddRoom(ruma::RoomId),
    Quit(bool),
    DelRoom(ruma::RoomId),
    SetSince(String),
    SendMessage(RoomId, String)
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct MessageInfo {
    pub message: String,
    pub words: Vec<String>,
    pub args: Vec<String>,
    pub sender: UserId,
    pub room_id: RoomId,
}
impl SonicBot {
    pub fn new(host: impl Into<String>, username: impl Into<String>, server_name: impl Into<String>, cleanup_on_ctrlc: bool, prefix: impl Into<String>, owner: impl Into<String>) -> Self {
        Self {
            data: StorageData::default(),
            host: host.into().trim_end_matches("/").to_string(),
            me: UserId::try_from(format!("@{}:{}",  username.into(), server_name.into()).as_str()).unwrap(),
            agent: AgentBuilder::new().timeout_read(Duration::from_secs(3)).build(),
            joined_rooms: Vec::new(),
            since: None,
            //last_event: EventResponse::default(),
            last_response_time: Instant::now(),
            starting: true,
            ctrlc_handler: ctrlc_handler::CtrlCHandler::new(),
            cleanup_on_ctrlc: cleanup_on_ctrlc,
            prefix: prefix.into(),
            owner: UserId::try_from(owner.into()).unwrap(),
            line_wrapper: None,
        }
    }
    fn login(&self, password: String) -> String {
        let content = json!({
            "type": "m.login.password",
            "identifier": {
                "type": "m.id.user",
                "user": self.me.localpart()
            },
            "password": password
        });
        let val: Value = self.post("login", content, None).unwrap().into_json().unwrap();
        self.get_tx().send(format!("{:?}", val)).unwrap();
        val["access_token"].as_str().unwrap().to_string()
    }
    fn post(&self, url_ext: impl Into<String>, content: Value, query_pairs: Option<Vec<(String, String)>>) -> Result<ureq::Response, ureq::Transport> {
        let mut r = self.agent.post(format!("{}{}{}", self.host, BASE, url_ext.into()).as_str()).set("Authorization", format!("Bearer {}", self.data.auth_token).as_str());
        if let Some(pairs) = query_pairs {
            for pair in pairs {
                r = r.query(pair.0.as_str(), pair.1.as_str());
            }
        }
        r.send_json(content).or_any_status()
    }
    fn put(&self, url_ext: impl Into<String>, content: Value, query_pairs: Option<Vec<(String, String)>>) -> Result<ureq::Response, ureq::Transport> {
        let mut r = self.agent.put(format!("{}{}{}", self.host, BASE, url_ext.into()).as_str()).set("Authorization", format!("Bearer {}", self.data.auth_token).as_str());
        if let Some(pairs) = query_pairs {
            for pair in pairs {
                r = r.query(pair.0.as_str(), pair.1.as_str());
            }
        }
        r.send_json(content).or_any_status()
    }
    fn get(&self, url_ext: impl Into<String>, query_pairs: Option<Vec<(String, String)>>) -> Result<ureq::Response, ureq::Transport> {
        let mut r = self.agent.get(format!("{}{}{}", self.host, BASE, url_ext.into()).as_str()).set("Authorization", format!("Bearer {}", self.data.auth_token).as_str());
        if let Some(pairs) = query_pairs {
            for pair in pairs {
                r = r.query(pair.0.as_str(), pair.1.as_str());
            }
        }
        r.call().or_any_status()
    }
    fn join_room_id(&self, room_id: ruma::RoomId) -> Value {
        //let room_string = room_id_or_alias.into();
        //let serv_name = server_name.into();
        let rid = room_id.to_string();
        let room = urlencoding::encode(rid.as_str());
        let content = json!({});
        self.post(format!("rooms/{}/join", room).as_str(), content, None).unwrap().into_json().unwrap()
    }
    fn sync(&self) -> Result<EventResponse, Value> {
        //let mut r = self.get("sync")
        let presence = self.calculate_presence();
        let req: ureq::Response;
        if let Some(start) = self.since.clone() {
            req = self.get("sync", Some(vec![("since".to_string(), start), ("full_state".to_string(), "false".to_string()), ("set_presence".to_string(), format!("{}", presence)), ("timeout".to_string(), "3000".to_string())])).unwrap();
        } else {
            req = self.get("sync", Some(vec![("full_state".to_string(), "true".to_string()), ("set_presence".to_string(), format!("{}", presence)), ("timeout".to_string(), "3000".to_string())])).unwrap();
        }
        let val: Value = req.into_json().unwrap();
        //self.get_tx().send(format!("{}", val.to_string())).unwrap();
        if val.clone().as_object().unwrap().contains_key("error") {
            Err(val)
        } else {
            Ok(serde_json::from_str::<Raw<EventResponse>>(val.to_string().as_str()).unwrap().deserialize().unwrap())
        }
    }
    fn calculate_presence(&self) -> PresenceState {
        if self.last_response_time.elapsed() > TIME_TO_IDLE {
            PresenceState::Unavailable
        } else {
            PresenceState::Online
        }
    }
    fn process_instructions(&mut self, instructions: Vec<Instructions>) -> Option<String> {
        if instructions.contains(&Instructions::Quit(false)) {
            std::process::exit(0);
        }
        for instruction in instructions {
            match instruction {
                Instructions::Quit(_x) => {
                    return Some("QUIT".to_string());
                },
                Instructions::DelRoom(x) => {
                    self.joined_rooms.retain(|z| z == &x);
                },
                Instructions::AddRoom(x) => {
                    if !self.joined_rooms.contains(&x) {
                        self.joined_rooms.push(x.clone());
                        self.join_room_id(x);
                    }
                },
                Instructions::UpdateLastResponseTime(x) => {
                    self.last_response_time = x;
                },
                Instructions::SetSince(x) => {
                    self.since = Some(x);
                },
                Instructions::SendMessage(room_id, message) => {
                    self.send_message(room_id, message);
                }
            }
        }
        None
    }
    #[cfg(not(target_os = "android"))]
    pub fn start(&mut self, password: impl Into<String>, room_ids: Vec<impl Into<String>>) {
        let (tx, rx) = channel::<String>();
        self.line_wrapper = Some(tx.clone());
        let mut line_wrapper = LineWrapper::new();
        let pass = password.into();
        self.data.auth_token = self.login(pass);
        let mut instructions: Vec<Instructions> = self.generate_instructions(self.sync().unwrap());
        let mut processed_instructions = self.process_instructions(instructions);
        self.starting = false;
        for this_room in room_ids {
            let this_room_string = this_room.into();
            let this_room_id_val: Value = self.get(format!("directory/room/{}", urlencoding::encode(this_room_string.as_str()).to_string().as_str()), None).unwrap().into_json().unwrap();
            let room_id = ruma::RoomId::try_from(this_room_id_val["room_id"].as_str().unwrap()).unwrap();
            if !self.joined_rooms.contains(&room_id) {
                self.join_room_id(room_id);
            }
        }
        while processed_instructions.is_none() && self.ctrlc_handler.should_continue() {
            instructions = self.generate_instructions(self.sync().unwrap());
            processed_instructions = self.process_instructions(instructions);
            Self::check_line_wrapper(&rx, &mut line_wrapper);
        }
    }
    fn get_tx(&self) -> Sender<String> {
        self.line_wrapper.clone().unwrap()
    }
    //#[cfg(target_os = "android")]
    //async fn generate_and_process_instructions(&mut self) {
    //    self.process_instructions(self.generate_instructions(self.sync().unwrap()));
    //}
    #[cfg(target_os = "android")]
    fn respond(&mut self, room_ids: Vec<impl Into<String>>) {
        //self.generate_and_process_instructions().await;
        //info!("[sonicbot-matrix] in _future");
        self.get_tx().send("test".to_string()).unwrap();
        self.process_instructions(self.generate_instructions(self.sync().unwrap()));
        //info!("[sonicbot-matrix] got past first instructions");
        self.starting = false;
        for this_room in room_ids {
            let this_room_string = this_room.into();
            let this_room_id_val: Value = self.get(format!("directory/room/{}", urlencoding::encode(this_room_string.as_str()).to_string().as_str()), None).unwrap().into_json().unwrap();
            let room_id = ruma::RoomId::try_from(this_room_id_val["room_id"].as_str().unwrap()).unwrap();
            if !self.joined_rooms.contains(&room_id) {
                self.join_room_id(room_id);
            }
        }
        loop {
            self.process_instructions(self.generate_instructions(self.sync().unwrap()));
        }
    } 
    #[cfg(target_os = "android")]
    pub async fn start(mut self, password: impl Into<String>, room_ids: Vec<String>) {
        let (tx, rx) = channel::<String>();
        self.line_wrapper = Some(tx.clone());
        let mut line_wrapper = LineWrapper::new();
        let pass = password.into();
        self.data.auth_token = self.login(pass);
        std::thread::spawn(move || {
            self.respond(room_ids); 
        });
        //info!("[sonicbot-matrix] got past _future");
        //self.generate_and_process_instructions().await;
        //self.starting = false;
        
        loop {
            //instructions = self.generate_instructions();
            //processed_instructions = self.process_instructions(instructions);
            //info!("[sonicbot-matrix] checking line_wrapper");
            Self::check_line_wrapper(&rx, &mut line_wrapper);
            //info!("[sonicbot-matrix] checked line_wrapper");
            line_wrapper.show_lines();
            //info!("[sonicbot-matrix] got past show_lines");
            next_frame().await;
        }
    }
    #[allow(unused_variables)]
    fn check_line_wrapper(rx: &Receiver<String>, line_wrapper: &mut LineWrapper) {
        if let Ok(message) = rx.try_recv() {
            linewrapper::lw_println!(line_wrapper, "{}", message);
        }
    }
    fn send_message(&self, room_id: RoomId, message: impl Into<String>) {
        let msg = message.into();
        let txid = Uuid::new_v4().to_simple().encode_lower(&mut Uuid::encode_buffer()).to_string();
        self.put(format!("rooms/{}/send/m.room.message/{}", room_id.clone(), txid).as_str(), json!({"body": msg, "msgtype": "m.text"}), None).unwrap();
    }
    fn generate_instructions(&self, event: EventResponse) -> Vec<Instructions> {
        let mut instructions: Vec<Instructions> = Vec::new();
        instructions.push(Instructions::SetSince(event.next_batch));
        //instructions.append()
        //if let Some(rooms) = event.rooms {
        handle_these_rooms!(self, instructions, RoomTypeData::Joined(event.rooms.join), RoomTypeData::Left(event.rooms.leave), RoomTypeData::Invited(event.rooms.invite));
        instructions
    }
}
