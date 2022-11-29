use model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::builder::CreateApplicationCommand;
use serenity::model;
use serenity::model::prelude::command::CommandOptionType;
use serenity::prelude::Context;

use crate::stats::{handle_top_interaction, handle_top_teamkills_interaction, handle_teamkillsbyhour_interaction, handle_top_suicides_interaction};

pub async fn run(ctx: Context, command: &ApplicationCommandInteraction) -> anyhow::Result<()> {
    let score_type = command
        .data
        .options
        .iter()
        .find(|elem| elem.name == "type")
        .and_then(|opt| opt.value.as_ref())
        .and_then(|num| num.as_str())
        .unwrap_or("unknown");

    match score_type {
        "type_score" => {
            handle_top_interaction(ctx, command).await?
        }
        "type_teamkills" => {
            handle_top_teamkills_interaction(ctx, command).await?
        }
        "type_teamkillbyhour" => {
            handle_teamkillsbyhour_interaction(ctx, command).await?
        }
        "type_suicides" => {
            handle_top_suicides_interaction(ctx, command).await?
        }
        _ => warn!("Unknown score type: {}", score_type),
    };

    Ok(())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("top")
        .description("Command to get top players in the server.")
        .create_option(|option| {
            option
                .name("type")
                .description("The type of stats")
                .kind(CommandOptionType::String)
                .required(true)
                .add_string_choice("Score", "type_score")
                .add_string_choice("Teamkills", "type_teamkills")
                .add_string_choice(
                    "TKH (players with less than 1 day playtime excluded)",
                    "type_teamkillbyhour",
                )
                .add_string_choice("Suicides", "type_suicides")
        })
        .create_option(|option| {
            option
                .name("count")
                .description("How many to show? (Default 10 and max 20)")
                .kind(CommandOptionType::Integer)
                .required(false)
        })
        .create_option(|option| {
            option
                .name("offset")
                .description("From which offset to show? (Default 0)")
                .kind(CommandOptionType::Integer)
                .required(false)
        })
}
