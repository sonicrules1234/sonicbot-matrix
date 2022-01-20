use crate::{Instructions, MessageArgs};

pub fn main(message_args: MessageArgs) -> Vec<Instructions> {
    let message_info = message_args.message_info;
    let mut instructions: Vec<Instructions> = Vec::new();
    instructions.push(Instructions::SendMessage(message_info.room_id, format!("{}: The following are my commands: {}.  For help using them use the help command.", message_info.sender, get_command_list().join(", "))));
    instructions
}

fn get_command_list() -> Vec<String> {
    let mut command_list: Vec<String> = Vec::new();
    for message_module in crate::essentials::on_message::get_essentials_on_message() {
        command_list.push(message_module.name);
    }
    for message_module in crate::plugins::on_message::get_plugins_on_message() {
        command_list.push(message_module.name);
    }
    command_list
}

pub fn help() -> String {
    String::from("list\nLists all the commands")
}
