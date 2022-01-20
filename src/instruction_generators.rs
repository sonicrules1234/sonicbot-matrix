use crate::{InvitedRoom, JoinedRoom, LeftRoom, Instructions, EventArgs};
use std::collections::BTreeMap;
#[derive(Debug, Clone)]
pub enum RoomTypeData {
    Joined(BTreeMap<ruma::RoomId, JoinedRoom>),
    Invited(BTreeMap<ruma::RoomId, InvitedRoom>),
    Left(BTreeMap<ruma::RoomId, LeftRoom>),
    //Knocked(BTreeMap<ruma::RoomId, KnockedRoom>),
}

pub fn handle_rooms(event_args: EventArgs) -> Vec<Instructions> {
    let mut instructions: Vec<Instructions> = Vec::new();
    let eoe = crate::essentials::on_event::get_essentials_on_event();
    for sonicbot_module in eoe {
        let mainfunction = sonicbot_module.main;
        instructions.append(&mut mainfunction(event_args.clone()));
    }
    instructions
}