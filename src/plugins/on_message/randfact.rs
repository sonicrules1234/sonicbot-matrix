use crate::{Instructions, MessageArgs};
use regex::Regex;
pub fn help() -> String {
    String::from("randfact\n Returns a random fact from http://randomfunfacts.com/")
}

pub fn main(message_args: MessageArgs) -> Vec<Instructions> {
    let message_info = message_args.message_info;
    let mut instructions: Vec<Instructions> = Vec::new();
    let data = ureq::get("http://randomfunfacts.com/").call().unwrap().into_string().unwrap();
    let matcher = Regex::new(r#"<strong><i>(.*)</i></strong>"#).unwrap();
    instructions.push(Instructions::SendMessage(message_info.room_id, format!("{}: {}", message_info.sender, matcher.captures(data.as_str()).unwrap()[1].to_string())));
    instructions
}