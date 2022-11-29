use model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::builder::CreateApplicationCommand;
use serenity::model;
use serenity::prelude::Context;

use crate::{vip::handle_vip_interaction};

pub async fn run(ctx: Context, command: &ApplicationCommandInteraction) -> anyhow::Result<()> {
    handle_vip_interaction(ctx, command).await?;
    Ok(())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("vip")
        .description("Command to get your VIP information in the server")
}
