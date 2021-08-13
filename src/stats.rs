use std::env;

use serenity::{client::Context, http::AttachmentType, model::interactions::{
        application_command::ApplicationCommandInteraction, InteractionResponseType,
    }};

use crate::{global_data::{DatabasePool, HandlebarsContext}, images::{generate_server_ranks_image, generate_server_suicides_image, generate_server_teamkills_image, generate_server_teamkillsbyhour_image}, models::{Count, PlayerScoreStats, PlayerTeamkillStats, ServerScoreTemplate, ServerTeamkillsTemplate}};

// TODO: Lots of duplicate code in this file
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

    let mut limit = command.data.options.iter()
        .find(|elem| elem.name == "count")
        .and_then(|opt| opt.value.as_ref())
        .and_then(|num| num.as_i64())
        .unwrap_or(10);

    limit = if limit < 1 { 10 } else { limit };
    limit = if limit > 20 { 20 } else { limit };

    let mut offset = command.data.options.iter()
        .find(|elem| elem.name == "offset")
        .and_then(|opt| opt.value.as_ref())
        .and_then(|num| num.as_i64())
        .unwrap_or(0);

    offset = if offset < 0 { 0 } else { offset };
    offset = if offset >= total_players { total_players - limit } else { offset };

    let data = sqlx::query_as!(
        PlayerScoreStats,
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

    let dir = env::current_dir()?;
    let template_data = ServerScoreTemplate {
        base_path: format!("file:///{}/templates/", dir.into_os_string().into_string().unwrap().replace('\\', "/")),
        players: data
    };

    let img = generate_server_ranks_image(handlebars, template_data)
        .await?;

    let msg_id = command
        .edit_original_interaction_response(&ctx.http, |response| {
            response.content(format!("Generating Score image..."))
        })
        .await?.id.0;

    command
        .edit_followup_message(&ctx.http, msg_id, |f| {
            f.content(format!("**Top Score** for positions {}-{}", offset + 1, offset + &limit)).add_file(AttachmentType::from((img.as_slice(), "top_score.png")))
        })
        .await?;

    Ok(())
}

pub async fn handle_top_teamkills_interaction(ctx: Context, command: ApplicationCommandInteraction) -> anyhow::Result<()> {
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

    let mut limit = command.data.options.iter()
        .find(|elem| elem.name == "count")
        .and_then(|opt| opt.value.as_ref())
        .and_then(|num| num.as_i64())
        .unwrap_or(10);

    limit = if limit < 1 { 10 } else { limit };
    limit = if limit > 20 { 20 } else { limit };

    let mut offset = command.data.options.iter()
        .find(|elem| elem.name == "offset")
        .and_then(|opt| opt.value.as_ref())
        .and_then(|num| num.as_i64())
        .unwrap_or(0);

    offset = if offset < 0 { 0 } else { offset };
    offset = if offset >= total_players { total_players - limit } else { offset };

    let data = sqlx::query_as!(
        PlayerScoreStats,
        "SELECT soldiername, FORMAT(score, '#,0') AS score, globalrank as global_rank, kills, deaths, tks as teamkills, suicide as suicides, FORMAT(kills / deaths, 2) AS kdr, (@row_number:=@row_number+1)+? AS position
        FROM tbl_playerstats AS ps
        INNER JOIN tbl_server_player AS sp ON ps.StatsID = sp.StatsID
        INNER JOIN tbl_playerdata AS pd ON sp.PlayerID = pd.PlayerID
        CROSS JOIN (SELECT @row_number:=0) AS t
        ORDER BY tks DESC
        LIMIT ? OFFSET ?",
        offset,
        limit,
        offset
    )
    .fetch_all(&pool)
    .await?;

    let dir = env::current_dir()?;
    let template_data = ServerScoreTemplate {
        base_path: format!("file:///{}/templates/", dir.into_os_string().into_string().unwrap().replace('\\', "/")),
        players: data
    };

    let img = generate_server_teamkills_image(handlebars, template_data)
        .await?;

    let msg_id = command
        .edit_original_interaction_response(&ctx.http, |response| {
            response.content(format!("Generating Teamkills image..."))
        })
        .await?.id.0;

    command
        .edit_followup_message(&ctx.http, msg_id, |f| {
            f.content(format!("**Top Teamkills** for positions {}-{}", offset + 1, offset + &limit)).add_file(AttachmentType::from((img.as_slice(), "top_teamkills.png")))
        })
        .await?;

    Ok(())
}

pub async fn handle_top_suicides_interaction(ctx: Context, command: ApplicationCommandInteraction) -> anyhow::Result<()> {
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

    let mut limit = command.data.options.iter()
        .find(|elem| elem.name == "count")
        .and_then(|opt| opt.value.as_ref())
        .and_then(|num| num.as_i64())
        .unwrap_or(10);

    limit = if limit < 1 { 10 } else { limit };
    limit = if limit > 20 { 20 } else { limit };

    let mut offset = command.data.options.iter()
        .find(|elem| elem.name == "offset")
        .and_then(|opt| opt.value.as_ref())
        .and_then(|num| num.as_i64())
        .unwrap_or(0);

    offset = if offset < 0 { 0 } else { offset };
    offset = if offset >= total_players { total_players - limit } else { offset };

    let data = sqlx::query_as!(
        PlayerScoreStats,
        "SELECT soldiername, FORMAT(score, '#,0') AS score, globalrank as global_rank, kills, deaths, tks as teamkills, suicide as suicides, FORMAT(kills / deaths, 2) AS kdr, (@row_number:=@row_number+1)+? AS position
        FROM tbl_playerstats AS ps
        INNER JOIN tbl_server_player AS sp ON ps.StatsID = sp.StatsID
        INNER JOIN tbl_playerdata AS pd ON sp.PlayerID = pd.PlayerID
        CROSS JOIN (SELECT @row_number:=0) AS t
        ORDER BY suicides DESC
        LIMIT ? OFFSET ?",
        offset,
        limit,
        offset
    )
    .fetch_all(&pool)
    .await?;

    let dir = env::current_dir()?;
    let template_data = ServerScoreTemplate {
        base_path: format!("file:///{}/templates/", dir.into_os_string().into_string().unwrap().replace('\\', "/")),
        players: data
    };

    let img = generate_server_suicides_image(handlebars, template_data)
        .await?;

    let msg_id = command
        .edit_original_interaction_response(&ctx.http, |response| {
            response.content(format!("Generating Suicides image..."))
        })
        .await?.id.0;

    command
        .edit_followup_message(&ctx.http, msg_id, |f| {
            f.content(format!("**Top Suicides** for positions {}-{}", offset + 1, offset + &limit)).add_file(AttachmentType::from((img.as_slice(), "top_suicides.png")))
        })
        .await?;

    Ok(())
}

pub async fn handle_teamkillsbyhour_interaction(ctx: Context, command: ApplicationCommandInteraction) -> anyhow::Result<()> {
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

    let total_players = sqlx::query_as!(Count, "SELECT COUNT(*) as count FROM tbl_playerstats WHERE playtime > 86400")
        .fetch_one(&pool)
        .await?
        .count;

    let mut limit = command.data.options.iter()
        .find(|elem| elem.name == "count")
        .and_then(|opt| opt.value.as_ref())
        .and_then(|num| num.as_i64())
        .unwrap_or(10);

    limit = if limit < 1 { 10 } else { limit };
    limit = if limit > 20 { 20 } else { limit };

    let mut offset = command.data.options.iter()
        .find(|elem| elem.name == "offset")
        .and_then(|opt| opt.value.as_ref())
        .and_then(|num| num.as_i64())
        .unwrap_or(0);

    offset = if offset < 0 { 0 } else { offset };
    offset = if offset >= total_players { total_players - limit } else { offset };

    let data = sqlx::query_as!(
        PlayerTeamkillStats,
        "SELECT soldiername, 
            FORMAT(score, '#,0') AS score,
            globalrank as global_rank,
            kills,
            deaths,
            tks as teamkills,
            suicide as suicides,
            FORMAT(kills / deaths, 2) AS kdr,
            (@row_number:=@row_number+1)+? AS position,
            date_format(from_unixtime(playtime), '%e d, %k h, %i m') AS playtime,
            FORMAT(tks / (playtime / 3600), 2) AS teamkills_by_hour
        FROM tbl_playerstats AS ps
        INNER JOIN tbl_server_player AS sp ON ps.StatsID = sp.StatsID
        INNER JOIN tbl_playerdata AS pd ON sp.PlayerID = pd.PlayerID
        CROSS JOIN (SELECT @row_number:=0) AS t
        WHERE playtime > 86400
        ORDER BY (tks / (playtime / 3600)) DESC
        LIMIT ? OFFSET ?",
        offset,
        limit,
        offset
    )
    .fetch_all(&pool)
    .await?;

    let dir = env::current_dir()?;
    let template_data = ServerTeamkillsTemplate {
        base_path: format!("file:///{}/templates/", dir.into_os_string().into_string().unwrap().replace('\\', "/")),
        players: data
    };

    let img = generate_server_teamkillsbyhour_image(handlebars, template_data)
        .await?;

    let msg_id = command
        .edit_original_interaction_response(&ctx.http, |response| {
            response.content(format!("Generating TKH image..."))
        })
        .await?.id.0;

    command
        .edit_followup_message(&ctx.http, msg_id, |f| {
            f.content(format!("**Top Teamkills By Hour** for positions {}-{}", offset + 1, offset + &limit)).add_file(AttachmentType::from((img.as_slice(), "top_tkh.png")))
        })
        .await?;

    Ok(())
}
