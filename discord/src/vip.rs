use serenity::{utils::Colour, model::{Timestamp, prelude::interaction::{application_command::ApplicationCommandInteraction, InteractionResponseType}}};
use chrono::prelude::*;
use serenity::{
    client::Context,
};

use crate::{
    global_data::{DatabasePool},
    models::{
        PlayerVip
    },
};

pub async fn handle_vip_interaction(
    ctx: Context,
    command: &ApplicationCommandInteraction,
) -> anyhow::Result<()> {
    command
        .create_interaction_response(&ctx.http, |response| {
            response.kind(InteractionResponseType::DeferredChannelMessageWithSource)
        })
        .await?;

    let dm_chan = command.user.create_dm_channel(&ctx).await.unwrap().id;

    let user_id = command.user.id.0;

    let pool = {
        let data_read = &ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    let msg_id = command
        .edit_original_interaction_response(&ctx.http, |response| {
            response.content("Fetching data...".to_string())
        })
        .await?
        .id
        .0;

    let vips = sqlx::query_as!(
        PlayerVip,
        "SELECT
                id,
                gametype,
                servergroup,
                playername,
                UNIX_TIMESTAMP(TIMESTAMP) AS timestamp,
                status,
                admin,
                comment,
                guid,
                discord_id,
                status like 'active' as active
            FROM
                vsm_vips
            WHERE
                discord_id LIKE ?",
        user_id
    )
    .fetch_all(&pool)
    .await?;

    let utc: DateTime<Utc> = Utc::now();
    if vips.is_empty() {
        dm_chan
            .send_message(&ctx, |m| {
                m.embed(|e| e
                    .title("No VIP found")
                    .description("No VIP status was detected for your user. You can donate via the link below. If you have already donated but haven't received your VIP, please contact an admin.\n\n**Donation link**\nhttps://www.g-portal.com/eur/donate/41292a1f7d32b66b33760f3a689902f2/LSDBF4\n\n**Pricing**\n5€ = 30 days\n8€ = 60 days\n10€ = 90 days")
                    .footer(|f| {
                        f
                        .text("Battlefield 4 - LSD")
                        .icon_url("https://cdn.discordapp.com/icons/390650889716760587/6c9e688f48ad2b897f5fd0ad750b09e4.webp?size=96");
                        f
                    })
                    .color(Colour::new(65535))
                    .timestamp(&Timestamp::from_unix_timestamp(utc.timestamp()).unwrap())
                    .author(|f| {
                        f
                        .name("Battlefield 4 - LSD")
                        .icon_url("https://cdn.discordapp.com/icons/390650889716760587/6c9e688f48ad2b897f5fd0ad750b09e4.webp?size=96")
                    })
                )
            })
            .await
            .unwrap();
    }
    else {
        dm_chan
            .say(&ctx, format!("Your VIP status at <t:{}>", utc.timestamp()))
            .await
            .unwrap();

        for vip in &vips {
            dm_chan
                .send_message(&ctx, |m| {
                    m.embed(|e| e
                        .footer(|f| {
                            f
                            .text("Battlefield 4 - LSD")
                            .icon_url("https://cdn.discordapp.com/icons/390650889716760587/6c9e688f48ad2b897f5fd0ad750b09e4.webp?size=96");
                            f
                        })
                        .field("Status", &vip.status, true)
                        .field(if &vip.active > &0 { "Expires" } else { "Expired" }, format!("<t:{}:R>", vip.timestamp.unwrap()), true)
                        .field("Extend VIP", "https://www.g-portal.com/eur/donate/41292a1f7d32b66b33760f3a689902f2/LSDBF4", false)
                        .color(if &vip.active > &0 { Colour::new(4321431) } else { Colour::new(16711680) })
                        .timestamp(&Timestamp::from_unix_timestamp(utc.timestamp()).unwrap())
                        .author(|f| {
                            f
                            .name(vip.playername.clone().unwrap_or("Unknown".to_string()))
                            .icon_url("https://cdn.discordapp.com/icons/390650889716760587/6c9e688f48ad2b897f5fd0ad750b09e4.webp?size=96")
                        })
                    )
                })
                .await
                .unwrap();
        }
    }

    if command.guild_id.is_none() {
        command
            .delete_followup_message(&ctx.http, msg_id)
            .await?;
    }
    else {
        command
            .edit_followup_message(&ctx.http, msg_id, |f| 
                f.content("Replied in DM")
            )
            .await?;
    }

    Ok(())
}
