//use ruma::api::client::r0::sync::sync_events::*;
use crate::{RoomAliasId, Instructions, MessageInfo, Instant, instruction_generators::RoomTypeData, EventArgs, MessageArgs};
use ruma::events::*;
//use std::collections::BTreeMap;

pub fn help() -> String {
    String::from("Runs on all syncs that have joined room updates.")
}

pub fn main<'a>(event_args: EventArgs<'a>) -> Vec<Instructions> {
    let mut instructions: Vec<Instructions> = Vec::new();
    if let RoomTypeData::Joined(ref x) = event_args.room_data {
        for (room_id, joined_room) in x.iter() {
            if event_args.ctrlc_handler.should_continue() {
                instructions.push(Instructions::AddRoom(room_id.clone()));
                for event in joined_room.state.events.clone() {
                    if let ruma::events::AnySyncStateEvent::RoomAliases(room_aliases_event) = event.deserialize().unwrap() {
                        let aliases = room_aliases_event.content.aliases;
                        for alias in aliases {
                            instructions.push(Instructions::SaveRoomAlias(room_id.clone(), alias));
                        }
                    } //else if let ruma::events::AnySyncStateEvent::RoomCanonicalAlias
                }
                if !event_args.starting {
                    for message_info in joined_room.timeline.events.iter().filter_map(|m| {
                        if let ruma::events::AnySyncRoomEvent::Message(message_event) = m.deserialize_as().unwrap() {
                            if let ruma::events::AnySyncMessageEvent::RoomMessage(room_message) = message_event {
                                let sender = room_message.sender;
                                if sender == event_args.me {
                                    return None;
                                }
                                if let room::message::MessageType::Text(text_message_event_content) = room_message.content.msgtype {
                                    let message = text_message_event_content.body;
                                    event_args.tx.send(format!("Got '{}' from '{}'", message, sender)).unwrap();
                                    if message.starts_with(&event_args.prefix) {
                                        let words: Vec<String> = message.split_whitespace().map(|w| w.to_string()).collect();
                                        let args = words[1..].to_vec();
                                        let mut room_aliases: Vec<RoomAliasId> = Vec::new();
                                        if event_args.room_to_aliases.contains_key(&room_id) {
                                            room_aliases.append(&mut event_args.room_to_aliases[&room_id].clone());
                                        }
                                        return Some(MessageInfo{
                                            message: message.clone(),
                                            words: words,
                                            args: args,
                                            sender: sender,
                                            room_id: room_id.clone(),
                                            room_aliases: room_aliases,
                                        });
                                    }
                                }
                            }
                        }
                        None
                    }) {
                        instructions.push(Instructions::UpdateLastResponseTime(Instant::now()));
                        //instructions.push(Instructions::RespondToMessage(message_info));
                        let eom = crate::essentials::on_message::get_essentials_on_message();
                        //let event_args_clone = event_args.clone();
                        for sonicbot_module in eom {
                            if message_info.words[0] == format!("{}{}", event_args.prefix.clone(), sonicbot_module.name).as_str() {
                                let event_args_clone = event_args.clone();
                                let mainfunction = sonicbot_module.main;
                                instructions.append(&mut mainfunction(MessageArgs::new(message_info.clone(), event_args_clone.owner, event_args_clone.ctrlc_handler, event_args_clone.cleanup_on_ctrlc, event_args.prefix.clone())));
                            }
                        }
                        let pom = crate::plugins::on_message::get_plugins_on_message();
                        for sonicbot_module in pom {
                            if message_info.words[0] == format!("{}{}", event_args.prefix.clone(), sonicbot_module.name).as_str() {
                                let event_args_clone = event_args.clone();
                                let mainfunction = sonicbot_module.main;
                                instructions.append(&mut mainfunction(MessageArgs::new(message_info.clone(), event_args_clone.owner, event_args_clone.ctrlc_handler, event_args_clone.cleanup_on_ctrlc, event_args.prefix.clone())));
                            }
                        }
                    }
                }
            } else {
                instructions.push(Instructions::Quit(event_args.cleanup_on_ctrlc));
                return instructions;
            }
        }
    }
    instructions
}
