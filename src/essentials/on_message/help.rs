use crate::{Instructions, MessageArgs};

pub fn main(message_args: MessageArgs) -> Vec<Instructions> {
    let message_info = message_args.message_info;
    let mut instructions: Vec<Instructions> = Vec::new();
    let words: Vec<String> = message_info.message.clone().split_whitespace().map(|x| x.to_string()).collect();
    if words.len() >= 2 {
        let command_name = words[1].clone();
        if let Some(help_string) = get_help_for_command(command_name.clone()) {
            instructions.push(Instructions::SendMessage(message_info.room_id.clone(), format!("{}: {}{}", message_info.sender.clone(), message_args.prefix.clone(), help_string.clone())));
        } else {
            instructions.push(Instructions::SendMessage(message_info.room_id.clone(), format!("{}: Couldn't find a command named '{}'", message_info.sender.clone(), command_name.clone())));
        }
    }
    instructions
}

fn get_help_for_command(command_name: String) -> Option<String> {
    for message_module in crate::essentials::on_message::get_essentials_on_message() {
        if message_module.name == command_name.clone() {
            return Some(message_module.help);
        }
    }
    for message_module in crate::plugins::on_message::get_plugins_on_message() {
        if command_name.clone() == message_module.name {
            return Some(message_module.help);
        }
    }
    None
}

pub fn help() -> String {
    String::from("help <command_name>\nDisplays information on what a command does and how to use it")
}
