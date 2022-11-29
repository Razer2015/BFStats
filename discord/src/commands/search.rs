use chrono::{DateTime, Utc};
use model::application::component::ButtonStyle;
use model::application::interaction::application_command::ApplicationCommandInteraction;
use model::application::interaction::InteractionResponseType;
use serenity::builder::{CreateApplicationCommand, CreateButton};
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::CommandDataOptionValue;
use serenity::model::{self, Timestamp};
use serenity::prelude::Context;
use serenity::utils::Colour;

use crate::battlelog::client::{ingame_metadata, search_user};

fn link_button(name: &str, link: &str) -> CreateButton {
    let mut b = CreateButton::default();
    b.label(name);
    b.style(ButtonStyle::Link);
    b.url(link);
    b
}

pub async fn run(ctx: Context, command: &ApplicationCommandInteraction) -> anyhow::Result<()> {
    command
        .create_interaction_response(&ctx.http, |response| {
            response.kind(InteractionResponseType::DeferredChannelMessageWithSource)
        })
        .await?;

    let option = command
        .data
        .options
        .get(0)
        .expect("Expected soldierName option")
        .resolved
        .as_ref()
        .expect("Expected soldierName object");

    if let CommandDataOptionValue::String(soldier) = option {
        let msg_id = command
            .edit_original_interaction_response(&ctx.http, |response| {
                response.content("Fetching the soldier from Battlelog...".to_string())
            })
            .await?
            .id
            .0;

        let fetch_result = search_user(soldier).await;
        if let Err(error) = fetch_result {
            error!("Error searching user from Battlelog: {}", error);
            command
                .edit_followup_message(&ctx.http, msg_id, |response| {
                    response.content(
                        "Error trying to fetch the user from Battlelog. Try again with more exact name.".to_string(),
                    )
                })
                .await?;
            return Ok(());
        };
        let soldier = fetch_result.unwrap();
        let profile_image = soldier.user.gravatar_md5.clone().map_or(
            "https://eaassets-a.akamaihd.net/battlelog/defaultavatars/default-avatar-204.png".to_string(),
            |md5| format!("https://secure.gravatar.com/avatar/{}?s=204&d=https://eaassets-a.akamaihd.net/battlelog/defaultavatars/default-avatar-204.png", md5)
        );

        let ingame_metadata = ingame_metadata(soldier.persona_id).await;
        let emblem_image = match ingame_metadata {
            Ok(metadata) => metadata.get_emblem_url().map_or(
                "https://eaassets-a.akamaihd.net/battlelog/prod/emblem/272/107/256/3370545042563659904.png"
                    .to_string(),
                |url| url,
            ),
            _ => {
                "https://eaassets-a.akamaihd.net/battlelog/prod/emblem/272/107/256/3370545042563659904.png"
                    .to_string()
            }
        };

        let utc: DateTime<Utc> = Utc::now();
        command
            .edit_followup_message(&ctx.http, msg_id, |f| {
                f.content("")
                    .embed(|embed| {
                        embed
                            .author(|a| {
                                a.name(&soldier.persona_name).icon_url(&profile_image).url(
                                    &format!(
                                        "https://battlelog.battlefield.com/bf4/user/{}/",
                                        soldier.persona_name
                                    ),
                                )
                            })
                            .image(&emblem_image)
                            .timestamp(&Timestamp::from_unix_timestamp(utc.timestamp()).unwrap())
                            .color(Colour::new(15790320))
                            .footer(|f| f.text("Battlefield 4 - LSD"))
                    })
                    .components(|c| {
                        c.create_action_row(|row| {
                            row.add_button(link_button(
                                "Battlelog",
                                &format!(
                                    "https://battlelog.battlefield.com/bf4/soldier/{}/stats/{}/pc/",
                                    soldier.persona_name, soldier.persona_id
                                ),
                            ))
                            .add_button(link_button(
                                "247FairPlay",
                                &format!(
                                    "https://www.247fairplay.com/CheatDetector/{}",
                                    soldier.persona_name
                                ),
                            ))
                            .add_button(link_button(
                                "BF4CR",
                                &format!(
                                    "https://bf4cheatreport.com/?pid={}&uid=&cnt=200&startdate=",
                                    soldier.persona_id
                                ),
                            ))
                            .add_button(link_button(
                                "BF4DB",
                                &format!("https://bf4db.com/player/{}", soldier.persona_id),
                            ))
                        })
                    })
            })
            .await?;
    } else {
        command
            .edit_original_interaction_response(&ctx.http, |response| {
                response.content(
                    "Player with the given name was not found from this server.".to_string(),
                )
            })
            .await?;
    }

    Ok(())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("search")
        .description("Get soldier information from Battlelog")
        .create_option(|option| {
            option
                .name("soldiername")
                .description("Name of the soldier (exact match)")
                .kind(CommandOptionType::String)
                .required(true)
        })
}
