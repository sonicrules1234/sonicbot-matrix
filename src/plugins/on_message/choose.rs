use rand::seq::SliceRandom;
use crate::{Instructions, MessageArgs};

pub fn help() -> String {
    String::from("choose <choices seperated by ' or ' without the quotes>\nChooses a random choice from <choices>")
}

pub fn main(message_args: MessageArgs) -> Vec<Instructions> {
    let message_info = message_args.message_info;
    let mut instructions: Vec<Instructions> = Vec::new();
    let msg = message_info.words[1..].join(" ");
    let choices: Vec<&str> = msg.split(" or ").collect();
    let choice = choices.choose(&mut rand::thread_rng()).unwrap().to_string();
    instructions.push(Instructions::SendMessage(message_info.room_id, format!("{}: {}", message_info.sender, choice)));
    instructions
}