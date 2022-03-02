use crate::{Instructions, instruction_generators::RoomTypeData, EventArgs, InvitedRoom};
use ruma::events::*;
//use std::collections::BTreeMap;

pub fn help() -> String {
    String::from("Runs on all syncs that have invited room updates.")
}
fn should_accept_invite(invited_room: InvitedRoom, event_args: EventArgs) -> bool {
    //println!("Got to should invite");
    for event in invited_room.invite_state.events {
        let stripped_state_event = event.deserialize_as::<AnyStrippedStateEvent>().unwrap();
        //println!("invite state_key = '{}' sender = '{}'", stripped_state_event.state_key(), stripped_state_event.sender());
        if stripped_state_event.state_key() == event_args.me {
            if stripped_state_event.sender() == &event_args.owner {
                //println!("Checking Event Type {:?}", stripped_state_event);
                if let AnyStrippedStateEvent::RoomMember(member_stripped_state_event) = stripped_state_event {
                    println!("membership = '{:?}'", member_stripped_state_event.content.membership);
                    if member_stripped_state_event.content.membership == room::member::MembershipState::Invite {
                        //println!("returning true");
                        return true;
                    }
                }
            }
        }
    }
    false
}
pub fn main<'a>(event_args: EventArgs<'a>) -> Vec<Instructions> {
    let mut instructions: Vec<Instructions> = Vec::new();
    //if !event_args.starting {
    if let RoomTypeData::Invited(ref x) = event_args.room_data {
        for (room_id, invited_room) in x.iter() {
            if event_args.ctrlc_handler.should_continue() && should_accept_invite(invited_room.clone(), event_args.clone()) {
                instructions.push(Instructions::AddRoom(room_id.clone()));
            }
        }
    }
    //}
    instructions
}
