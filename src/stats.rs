use std::env;

use serenity::{client::Context, http::AttachmentType, model::interactions::{
        application_command::ApplicationCommandInteraction, InteractionResponseType,
    }};

use crate::{global_data::{DatabasePool, HandlebarsContext}, images::generate_server_ranks_image, models::{Count, PlayerStats, ServerRankTemplate}};

pub async fn handle_top_interaction(ctx: Context, command: ApplicationCommandInteraction) -> anyhow::Result<()> {
    command
        .create_interaction_response(&ctx.http, |response| {
            response.kind(InteractionResponseType::DeferredChannelMessageWithSource)
        })
        .await?;
    
    let pool = {
        let data_read = &ctx.data.read().await;
        data_read.get::<DatabasePool>().unwrap().clone()
    };

    let handlebars = {
        let data_read = &ctx.data.read().await;
        data_read.get::<HandlebarsContext>().unwrap().clone()
    };

    let total_players = sqlx::query_as!(Count, "SELECT COUNT(*) as count FROM tbl_playerstats")
        .fetch_one(&pool)
        .await?
        .count;

    let msg_id = command
        .edit_original_interaction_response(&ctx.http, |response| {
            response.content(format!("Total players {}", total_players))
        })
        .await?.id.0;

    let limit = command.data.options.iter()
        .find(|elem| elem.name == "count")
        .and_then(|opt| opt.value.as_ref())
        .and_then(|num| num.as_i64())
        .unwrap_or(10);

    let offset = command.data.options.iter()
        .find(|elem| elem.name == "offset")
        .and_then(|opt| opt.value.as_ref())
        .and_then(|num| num.as_i64())
        .unwrap_or(0);

    let data = sqlx::query_as!(
        PlayerStats,
        "SELECT soldiername, FORMAT(score, '#,0') AS score, globalrank as global_rank, kills, deaths, tks as teamkills, suicide as suicides, FORMAT(kills / deaths, 2) AS kdr, (@row_number:=@row_number+1)+? AS position
        FROM tbl_playerstats AS ps
        INNER JOIN tbl_server_player AS sp ON ps.StatsID = sp.StatsID
        INNER JOIN tbl_playerdata AS pd ON sp.PlayerID = pd.PlayerID
        CROSS JOIN (SELECT @row_number:=0) AS t
        ORDER BY rankScore
        LIMIT ? OFFSET ?",
        offset,
        limit,
        offset
    )
    .fetch_all(&pool)
    .await?;

    let dir = env::current_dir().unwrap();
    let template_data = ServerRankTemplate {
        base_path: format!("file:///{}/templates/", dir.into_os_string().into_string().unwrap().replace('\\', "/")),
        players: data
    };

    let img = generate_server_ranks_image(handlebars, template_data)
        .await?;

    command
        .edit_followup_message(&ctx.http, msg_id, |f| {
            f.content(format!("Total players {}", total_players)).add_file(AttachmentType::from((img.as_slice(), "top.png")))
        })
        .await?;

    Ok(())
}
