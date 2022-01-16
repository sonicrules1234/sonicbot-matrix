//use sonic_serde_object::SonicSerdeObject;
use ruma::RoomId;
use ruma::UserId;
use ruma::UInt;
use uuid::Uuid;
use ruma::DeviceKeyAlgorithm;
use ruma::{presence::PresenceState, serde::Raw, api::client::r0::sync::sync_events::*, events::*};
use std::collections::BTreeMap;
use ureq::{Agent, AgentBuilder, OrAnyStatus};
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};
use std::time::{Instant, Duration};
//use std::thread;
//mod instruction_generators;
//use std::sync::mpsc::{channel, Sender, Receiver, TryRecvError};
//use std::thread::Duration;
const BASE: &str = "/_matrix/client/v3/";
const TIME_TO_IDLE: Duration = Duration::from_secs(3 * 60);
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct StorageData {
    pub auth_token: String,
}
/*
pub enum Presence {
    Online,
    Offline,
    Unavailable,
}

impl std::fmt::Display for Presence {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let output = match self {
            Presence::Online => "online",
            Presence::Offline => "offline",
            Presence::Unavailable => "unavailable",
        };
        write!(f, "{}", output)
    }
}


struct RoomUserData {
    name: String,
    
}

struct RoomData {
    id: String,
    local_aliases: Vec<String>,
    global_aliases: Vec<String>,
    user_data: BTreeMap<String, RoomUserData>,
}

enum MatrixRoomEvent {
    CannonicalAlias(Value),
    Create(Value),
    JoinRules(Value),
    Member(Value),
    PowerLevels(Value),

}
*/

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct EventResponse {
    pub next_batch: String,
    pub rooms: Option<Rooms>,
    pub presence: Option<Presence>,
    pub account_data: Option<GlobalAccountData>,
    pub to_device: Option<ToDevice>,
    pub device_lists: Option<DeviceLists>,
    pub device_one_time_keys_count: Option<BTreeMap<DeviceKeyAlgorithm, UInt>>,
}

#[derive(Debug)]
pub struct SonicBot {
    data: StorageData,
    host: String,
    username: String,
    server_name: String,
    agent: Agent,
    joined_rooms: Vec<ruma::RoomId>,
    since: Option<String>,
    //last_event: EventResponse,
    last_response_time: Instant,
    starting: bool,
    ctrlc_handler: ctrlc_handler::CtrlCHandler,
    cleanup_on_ctrlc: bool,
}
#[derive(PartialEq, Eq, Debug, Clone)]
enum Instructions {
    UpdateLastResponseTime(Instant),
    AddRoom(ruma::RoomId),
    Quit(bool),
    DelRoom(ruma::RoomId),
    SetSince(String),
    RespondToMessage(MessageInfo)
}

#[derive(PartialEq, Eq, Debug, Clone)]
struct MessageInfo {
    message: String,
    sender: UserId,
    room_id: RoomId,
}
impl SonicBot {
    pub fn new(host: impl Into<String>, username: impl Into<String>, server_name: impl Into<String>, cleanup_on_ctrlc: bool) -> Self {
        Self {
            data: StorageData::default(),
            host: host.into().trim_end_matches("/").to_string(),
            username: username.into(),
            server_name: server_name.into(),
            agent: AgentBuilder::new().timeout_read(Duration::from_secs(3)).build(),
            joined_rooms: Vec::new(),
            since: None,
            //last_event: EventResponse::default(),
            last_response_time: Instant::now(),
            starting: true,
            ctrlc_handler: ctrlc_handler::CtrlCHandler::new(),
            cleanup_on_ctrlc: cleanup_on_ctrlc,
        }
    }
    /*
    fn my_middleware(request: Request, next: MiddlewareNext) -> Result<Response, ureq::Error> {
        let mut req = request.clone();
        // do middleware things
        req.timeout_response();
        // continue the middleware chain
        next.handle(req)
    }
    */
    fn login(&self, password: String) -> String {
        let content = json!({
            "type": "m.login.password",
            "identifier": {
                "type": "m.id.user",
                "user": self.username
            },
            "password": password
        });
        let val: Value = self.post("login", content, None).unwrap().into_json().unwrap();
        //let val: Value = self.agent.post(format!("{}{}/login", self.host, BASE).as_str()).send_json(content).unwrap().into_json().unwrap();
        println!("{:?}", val);
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
    fn get_joined_rooms(&self) -> Vec<String> {
        let val: Value = self.get("joined_rooms", None).unwrap().into_json().unwrap();
        val["joined_rooms"].as_array().unwrap().iter().map(|x| x.to_string()).collect()
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
        println!("{}", val.to_string());
        if val.clone().as_object().unwrap().contains_key("error") {
            Err(val)
        } else {
            //Ok(serde_json::from_str::<Raw<EventResponse>>(val.to_string().as_str()).unwrap().deserialize().unwrap())
            Ok(serde_json::from_value(val).unwrap())
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
                        self.joined_rooms.push(x);
                    }
                },
                Instructions::UpdateLastResponseTime(x) => {
                    self.last_response_time = x;
                },
                Instructions::SetSince(x) => {
                    self.since = Some(x);
                },
                Instructions::RespondToMessage(x) => {
                    self.respond_to_message(x);
                }
            }
        }
        None
    }
    pub fn start(&mut self, password: impl Into<String>, room_ids: Vec<impl Into<String>>) {
        let pass = password.into();
        self.data.auth_token = self.login(pass);
        let mut instructions: Vec<Instructions> = self.generate_instructions(self.sync().unwrap());
        let mut processed_instructions = self.process_instructions(instructions);
        self.starting = false;
        for this_room in room_ids {
            let this_room_string = this_room.into();
            let this_room_id_val: Value = self.get(format!("directory/room/{}", urlencoding::encode(this_room_string.as_str()).to_string().as_str()), None).unwrap().into_json().unwrap();
            println!("{:#?}", this_room_id_val);
            let room_id = ruma::RoomId::try_from(this_room_id_val["room_id"].as_str().unwrap()).unwrap();
            if !self.joined_rooms.contains(&room_id) {
                self.join_room_id(room_id);
            }
        }
        while processed_instructions.is_none() && self.ctrlc_handler.should_continue() {
            instructions = self.generate_instructions(self.sync().unwrap());
            processed_instructions = self.process_instructions(instructions);
        }
    }
    fn respond_to_message(&mut self, message_info: MessageInfo) {
        let sender = message_info.sender;
        let message = message_info.message;
        let room_id = message_info.room_id;
        if sender != UserId::try_from(format!("@{}:{}", self.username, self.server_name).as_str()).unwrap() {
            if message.starts_with("!hi") {
                self.send_message(room_id.to_string(), format!("{}: Hello!", sender.localpart()));
            }
        }
    }
    fn send_message(&self, room_id: String, message: impl Into<String>) {
        let msg = message.into();
        let txid = Uuid::new_v4().to_simple().encode_lower(&mut Uuid::encode_buffer()).to_string();
        self.put(format!("rooms/{}/send/m.room.message/{}", room_id.clone(), txid).as_str(), json!({"body": msg, "msgtype": "m.text"}), None).unwrap();
    }
    fn generate_instructions(&self, event: EventResponse) -> Vec<Instructions> {
        let mut instructions: Vec<Instructions> = Vec::new();
        instructions.push(Instructions::SetSince(event.next_batch));
        //instructions.append()
        if let Some(rooms) = event.rooms {
            for (room_id, joined_room) in rooms.join.iter() {
                if self.ctrlc_handler.should_continue() {
                    instructions.push(Instructions::AddRoom(room_id.clone()));
                    if !self.starting {
                        for message_info in joined_room.timeline.events.iter().filter_map(|m| {
                            if let ruma::events::AnySyncRoomEvent::Message(message_event) = m.deserialize_as().unwrap() {
                                if let AnySyncMessageEvent::RoomMessage(room_message) = message_event {
                                    let sender = room_message.sender;
                                    if let room::message::MessageType::Text(text_message_event_content) = room_message.content.msgtype {
                                        return Some(MessageInfo{
                                            message: text_message_event_content.body,
                                            sender: sender,
                                            room_id: room_id.clone(),
                                        });
                                    }
                                }
                            }
                            None
                        }) {
                            instructions.push(Instructions::UpdateLastResponseTime(Instant::now()));
                            instructions.push(Instructions::RespondToMessage(message_info));
                        }
                    }
                } else {
                    instructions.push(Instructions::Quit(self.cleanup_on_ctrlc));
                    return instructions;
                }
            }
        }
        instructions
    }
}
