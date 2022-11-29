use model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::builder::CreateApplicationCommand;
use serenity::model;
use serenity::model::prelude::command::CommandOptionType;
use serenity::prelude::Context;

use crate::{stats::{handle_rank_interaction}, models::Server};

pub async fn run(ctx: Context, command: &ApplicationCommandInteraction) {
    if let Err(why) = handle_rank_interaction(ctx, command).await {
        println!("Error: {}", why)
    };
}

fn no_space(x : String) -> String{
    x.replace(' ', "-")
}

pub fn register(command: &mut CreateApplicationCommand, servers: Vec<Server>) -> &mut CreateApplicationCommand {
    if servers.len() > 1 { // Multiple servers
        command
            .name("rank")
            .description("Command to get your rank in the server.")
            .create_option(|option| {
                let opt = option
                    .name("server")
                    .description("Which server you want the stats from?")
                    .kind(CommandOptionType::String)
                    .required(true);

                for (_pos, server) in servers.iter().enumerate() {
                    opt.add_string_choice(no_space(server.server_name.as_ref().unwrap_or(&"Unknown".to_string()).to_string()).trim(), server.server_id);
                }

                option
            })
            .create_option(|option| {
                option
                    .name("soldiername")
                    .description("Name of the soldier (can be partial but must have only one match).")
                    .kind(CommandOptionType::String)
                    .required(true)
            })
    }
    else {
        command
            .name("rank")
            .description("Command to get your rank in the server.")
            .create_option(|option| {
                option
                    .name("soldiername")
                    .description("Name of the soldier (can be partial but must have only one match).")
                    .kind(CommandOptionType::String)
                    .required(true)
        })
    }
}
