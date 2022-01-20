//use ruma::api::client::r0::sync::sync_events::*;
use crate::{Instructions, instruction_generators::RoomTypeData, EventArgs};
//use std::collections::BTreeMap;

pub fn help() -> String {
    String::from("Runs on all syncs that have left rooms states")
}

pub fn main(event_args: EventArgs) -> Vec<Instructions> {
    let mut instructions: Vec<Instructions> = Vec::new();
    if let RoomTypeData::Left(ref x) = event_args.room_data {
        for (room_id, _left_room) in x.iter() {
            instructions.push(Instructions::DelRoom(room_id.clone()))
        }
    }
    instructions
}
