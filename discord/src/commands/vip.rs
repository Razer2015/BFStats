use model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::builder::CreateApplicationCommand;
use serenity::model;
use serenity::prelude::Context;

use crate::{vip::handle_vip_interaction};

pub async fn run(ctx: Context, command: &ApplicationCommandInteraction) {
    if let Err(why) = handle_vip_interaction(ctx, command).await {
        println!("Error: {}", why)
    };
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("vip")
        .description("Command to get your VIP information in the server")
}
