use rand::Rng;
use serenity::{
    client::Context,
    model::{interactions::{
        application_command::ApplicationCommandInteraction, InteractionResponseType,
    }, prelude::AttachmentType},
};

use crate::{battlelog::search_user, global_data::{DatabasePool, HandlebarsContext}, images::{generate_player_rank_image, generate_server_ranks_image, generate_server_suicides_image, generate_server_teamkills_image, generate_server_teamkillsbyhour_image}, models::{Count, PlayerData, PlayerScoreStats, PlayerTeamkillStats, Server, ServerRankTemplate, ServerScoreTemplate, ServerTeamkillsTemplate}};

// TODO: Lots of duplicate code in this file
pub async fn handle_top_interaction(
    ctx: Context,
    command: ApplicationCommandInteraction,
) -> anyhow::Result<()> {
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

    let mut limit = command
        .data
        .options
        .iter()
        .find(|elem| elem.name == "count")
        .and_then(|opt| opt.value.as_ref())
        .and_then(|num| num.as_i64())
        .unwrap_or(10);

    limit = if limit < 1 { 10 } else { limit };
    limit = if limit > 20 { 20 } else { limit };

    let mut offset = command
        .data
        .options
        .iter()
        .find(|elem| elem.name == "offset")
        .and_then(|opt| opt.value.as_ref())
        .and_then(|num| num.as_i64())
        .unwrap_or(0);

    offset = if offset < 0 { 0 } else { offset };
    offset = if offset >= total_players {
        total_players - limit
    } else {
        offset
    };

    let data = sqlx::query_as!(
        PlayerScoreStats,
        "SELECT soldiername, 
            FORMAT(score, '#,0') AS score, 
            globalrank as global_rank, 
            kills, 
            deaths, 
            tks as teamkills, 
            suicide as suicides, 
            FORMAT(kills / deaths, 2) AS kdr, 
            (@row_number:=@row_number+1)+? AS position, 
            CONCAT(FLOOR(playtime * 0.00027777777777778), 'h ', MINUTE(from_unixtime(playtime)), 'm') AS playtime
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

    let template_data = ServerScoreTemplate {
        base_path: format!(
            "{}public/",
            dotenv::var("IMAGEAPI_URL").unwrap_or("http://localhost:3000/".to_string())
        ),
        players: data,
    };

    let img = generate_server_ranks_image(handlebars, template_data).await?;

    let msg_id = command
        .edit_original_interaction_response(&ctx.http, |response| {
            response.content("Generating Score image...".to_string())
        })
        .await?
        .id
        .0;

    command
        .edit_followup_message(&ctx.http, msg_id, |f| {
            f.content(format!(
                "**Top Score** for positions {}-{}",
                offset + 1,
                offset + &limit
            ))
            .add_file(AttachmentType::from((img.as_slice(), "top_score.png")))
        })
        .await?;

    Ok(())
}

pub async fn handle_top_teamkills_interaction(
    ctx: Context,
    command: ApplicationCommandInteraction,
) -> anyhow::Result<()> {
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

    let mut limit = command
        .data
        .options
        .iter()
        .find(|elem| elem.name == "count")
        .and_then(|opt| opt.value.as_ref())
        .and_then(|num| num.as_i64())
        .unwrap_or(10);

    limit = if limit < 1 { 10 } else { limit };
    limit = if limit > 20 { 20 } else { limit };

    let mut offset = command
        .data
        .options
        .iter()
        .find(|elem| elem.name == "offset")
        .and_then(|opt| opt.value.as_ref())
        .and_then(|num| num.as_i64())
        .unwrap_or(0);

    offset = if offset < 0 { 0 } else { offset };
    offset = if offset >= total_players {
        total_players - limit
    } else {
        offset
    };

    let data = sqlx::query_as!(
        PlayerScoreStats,
        "SELECT soldiername, 
            FORMAT(score, '#,0') AS score, 
            globalrank as global_rank, 
            kills, 
            deaths, 
            tks as teamkills, 
            suicide as suicides, 
            FORMAT(kills / deaths, 2) AS kdr, 
            (@row_number:=@row_number+1)+? AS position, 
            CONCAT(FLOOR(playtime * 0.00027777777777778), 'h ', MINUTE(from_unixtime(playtime)), 'm') AS playtime
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

    let template_data = ServerScoreTemplate {
        base_path: format!(
            "{}public/",
            dotenv::var("IMAGEAPI_URL").unwrap_or("http://localhost:3000/".to_string())
        ),
        players: data,
    };

    let img = generate_server_teamkills_image(handlebars, template_data).await?;

    let msg_id = command
        .edit_original_interaction_response(&ctx.http, |response| {
            response.content("Generating Teamkills image...".to_string())
        })
        .await?
        .id
        .0;

    command
        .edit_followup_message(&ctx.http, msg_id, |f| {
            f.content(format!(
                "**Top Teamkills** for positions {}-{}",
                offset + 1,
                offset + &limit
            ))
            .add_file(AttachmentType::from((img.as_slice(), "top_teamkills.png")))
        })
        .await?;

    Ok(())
}

pub async fn handle_top_suicides_interaction(
    ctx: Context,
    command: ApplicationCommandInteraction,
) -> anyhow::Result<()> {
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

    let mut limit = command
        .data
        .options
        .iter()
        .find(|elem| elem.name == "count")
        .and_then(|opt| opt.value.as_ref())
        .and_then(|num| num.as_i64())
        .unwrap_or(10);

    limit = if limit < 1 { 10 } else { limit };
    limit = if limit > 20 { 20 } else { limit };

    let mut offset = command
        .data
        .options
        .iter()
        .find(|elem| elem.name == "offset")
        .and_then(|opt| opt.value.as_ref())
        .and_then(|num| num.as_i64())
        .unwrap_or(0);

    offset = if offset < 0 { 0 } else { offset };
    offset = if offset >= total_players {
        total_players - limit
    } else {
        offset
    };

    let data = sqlx::query_as!(
        PlayerScoreStats,
        "SELECT soldiername, 
            FORMAT(score, '#,0') AS score, 
            globalrank as global_rank, 
            kills, 
            deaths, 
            tks as teamkills, 
            suicide as suicides, 
            FORMAT(kills / deaths, 2) AS kdr, 
            (@row_number:=@row_number+1)+? AS position,
            CONCAT(FLOOR(playtime * 0.00027777777777778), 'h ', MINUTE(from_unixtime(playtime)), 'm') AS playtime
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

    let template_data = ServerScoreTemplate {
        base_path: format!(
            "{}public/",
            dotenv::var("IMAGEAPI_URL").unwrap_or("http://localhost:3000/".to_string())
        ),
        players: data,
    };

    let img = generate_server_suicides_image(handlebars, template_data).await?;

    let msg_id = command
        .edit_original_interaction_response(&ctx.http, |response| {
            response.content("Generating Suicides image...".to_string())
        })
        .await?
        .id
        .0;

    command
        .edit_followup_message(&ctx.http, msg_id, |f| {
            f.content(format!(
                "**Top Suicides** for positions {}-{}",
                offset + 1,
                offset + &limit
            ))
            .add_file(AttachmentType::from((img.as_slice(), "top_suicides.png")))
        })
        .await?;

    Ok(())
}

pub async fn handle_teamkillsbyhour_interaction(
    ctx: Context,
    command: ApplicationCommandInteraction,
) -> anyhow::Result<()> {
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

    let total_players = sqlx::query_as!(
        Count,
        "SELECT COUNT(*) as count FROM tbl_playerstats WHERE playtime > 86400"
    )
    .fetch_one(&pool)
    .await?
    .count;

    let mut limit = command
        .data
        .options
        .iter()
        .find(|elem| elem.name == "count")
        .and_then(|opt| opt.value.as_ref())
        .and_then(|num| num.as_i64())
        .unwrap_or(10);

    limit = if limit < 1 { 10 } else { limit };
    limit = if limit > 20 { 20 } else { limit };

    let mut offset = command
        .data
        .options
        .iter()
        .find(|elem| elem.name == "offset")
        .and_then(|opt| opt.value.as_ref())
        .and_then(|num| num.as_i64())
        .unwrap_or(0);

    offset = if offset < 0 { 0 } else { offset };
    offset = if offset >= total_players {
        total_players - limit
    } else {
        offset
    };

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

    let template_data = ServerTeamkillsTemplate {
        base_path: format!(
            "{}public/",
            dotenv::var("IMAGEAPI_URL").unwrap_or("http://localhost:3000/".to_string())
        ),
        players: data,
    };

    let img = generate_server_teamkillsbyhour_image(handlebars, template_data).await?;

    let msg_id = command
        .edit_original_interaction_response(&ctx.http, |response| {
            response.content("Generating TKH image...".to_string())
        })
        .await?
        .id
        .0;

    command
        .edit_followup_message(&ctx.http, msg_id, |f| {
            f.content(format!(
                "**Top Teamkills By Hour** for positions {}-{}",
                offset + 1,
                offset + &limit
            ))
            .add_file(AttachmentType::from((img.as_slice(), "top_tkh.png")))
        })
        .await?;

    Ok(())
}

pub async fn handle_rank_interaction(
    ctx: Context,
    command: ApplicationCommandInteraction,
) -> anyhow::Result<()> {
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

    let server = match command
        .data
        .options
        .iter()
        .find(|elem| elem.name == "server")
        .and_then(|opt| opt.value.as_ref())
        .and_then(|num| num.as_i64()) {
            Some(serverid) => {
                sqlx::query_as!(Server, "SELECT serverid as server_id, servername as server_name FROM tbl_server WHERE serverid = ?", serverid)
                    .fetch_one(&pool)
                    .await
            },
            None => {
                sqlx::query_as!(Server, "SELECT serverid as server_id, servername as server_name FROM tbl_server LIMIT 1")
                    .fetch_one(&pool)
                    .await
            }
        };

    if let Err(why) = server {
        println!("Couldn't find the server {}", why);

        command
        .edit_original_interaction_response(&ctx.http, |response| {
            response.content("Error finding the server.".to_string())
        })
        .await?;

        return Ok(());
    };

    let server = server.unwrap();
    let total_players = sqlx::query_as!(Count, "SELECT COUNT(*) as count FROM tbl_server_player WHERE serverid = ?", server.server_id)
        .fetch_one(&pool)
        .await?
        .count;

    let soldiername = command
        .data
        .options
        .iter()
        .find(|elem| elem.name == "soldiername")
        .and_then(|opt| opt.value.as_ref())
        .and_then(|num| num.as_str())
        .unwrap_or("");

    let soldiers = sqlx::query_as!(
        PlayerData,
            "SELECT
                soldiername,
                clantag as clan_tag,
                pd.playerid as player_id,
                FORMAT(score, '#,0') AS score,
                globalrank as global_rank,
                kills,
                deaths,
                rankscore as rank_score,
                wins,
                losses,
                headshots,
                FORMAT(highscore, '#,0') AS highscore,
                deathstreak,
                killstreak,
                tks as teamkills,
                suicide as suicides,
                FORMAT(kills / deaths, 2) AS kdr,
                CONCAT(FLOOR(playtime * 0.00027777777777778), 'h ', MINUTE(from_unixtime(playtime)), 'm') AS playtime,
                rounds 
            FROM
                tbl_playerstats AS ps 
                INNER JOIN
                tbl_server_player AS sp 
                ON ps.StatsID = sp.StatsID 
                INNER JOIN
                tbl_playerdata AS pd 
                ON sp.PlayerID = pd.PlayerID 
            WHERE
                soldiername LIKE ? AND serverid = ?
            UNION
            SELECT
                soldiername,
                clantag as clan_tag,
                pd.playerid as player_id,
                FORMAT(score, '#,0') AS score,
                globalrank as global_rank,
                kills,
                deaths,
                rankscore as rank_score,
                wins,
                losses,
                headshots,
                FORMAT(highscore, '#,0') AS highscore,
                deathstreak,
                killstreak,
                tks as teamkills,
                suicide as suicides,
                FORMAT(kills / deaths, 2) AS kdr,
                CONCAT(FLOOR(playtime * 0.00027777777777778), 'h ', MINUTE(from_unixtime(playtime)), 'm') AS playtime,
                rounds 
            FROM
                tbl_playerstats AS ps 
                INNER JOIN
                tbl_server_player AS sp 
                ON ps.StatsID = sp.StatsID 
                INNER JOIN
                tbl_playerdata AS pd 
                ON sp.PlayerID = pd.PlayerID 
            WHERE
                NOT EXISTS 
                (
                SELECT
                    * 
                FROM
                    tbl_playerdata 
                WHERE
                    soldiername LIKE ?
                )
                AND soldiername LIKE ? AND serverid = ? LIMIT 2",
            soldiername,
            server.server_id,
            soldiername,
            format!("%{}%", soldiername),
            server.server_id
        )
        .fetch_all(&pool)
        .await?;

    if soldiers.len() != 1 {
        command
            .edit_original_interaction_response(&ctx.http, |response| {
                response.content("Player with the given name was not found from this server.".to_string())
            })
            .await?;

        return Ok(());
    }

    let msg_id = command
        .edit_original_interaction_response(&ctx.http, |response| {
            response.content("Fetching the soldier from Battlelog...".to_string())
        })
        .await?.id.0;

    let db_soldier = soldiers.get(0).unwrap();
    let soldier_name = db_soldier.soldiername.as_ref().unwrap();
    let soldier = search_user(soldier_name).await;
    let profile_image = match soldier {
        Ok(user) => user.user.gravatar_md5.map_or(
            "https://eaassets-a.akamaihd.net/battlelog/defaultavatars/default-avatar-204.png".to_string(),
            |md5| format!("https://secure.gravatar.com/avatar/{}?s=204&d=https://eaassets-a.akamaihd.net/battlelog/defaultavatars/default-avatar-204.png", md5)
        ),
        _ => "https://eaassets-a.akamaihd.net/battlelog/defaultavatars/default-avatar-204.png".to_string(),
    };

    let bg_index = {
        let mut rng = rand::thread_rng();
        rng.gen_range(1..11)
    };
    let template_data = ServerRankTemplate {
        base_path: format!("{}public/", dotenv::var("IMAGEAPI_URL").unwrap_or("http://localhost:3000/".to_string())),
        servername: server.server_name,
        total_players,
        profile_image_url: profile_image,
        bg_index,
        clan_tag: db_soldier.clan_tag.to_owned(),
        soldiername: db_soldier.soldiername.to_owned(),
        rank_score: db_soldier.rank_score,
        score: db_soldier.score.to_owned(),
        global_rank: db_soldier.global_rank,
        kills: db_soldier.kills,
        deaths: db_soldier.deaths,
        teamkills: db_soldier.teamkills,
        suicides: db_soldier.suicides,
        wins: db_soldier.wins,
        losses: db_soldier.losses,
        headshots: db_soldier.headshots,
        highscore: db_soldier.highscore.to_owned(),
        killstreak: db_soldier.killstreak,
        deathstreak: db_soldier.deathstreak,
        kdr: db_soldier.kdr.to_owned(),
        playtime: db_soldier.playtime.to_owned(),
        rounds: db_soldier.rounds
    };

    command
        .edit_followup_message(&ctx.http, msg_id, |f| {
            f.content("Generating Rank image...".to_string())
        })
        .await?;

    let img = generate_player_rank_image(handlebars, template_data)
        .await?;

    command
        .edit_followup_message(&ctx.http, msg_id, |f| {
            f.content(format!("**Rank** for **{}**", db_soldier.soldiername.as_ref().unwrap_or(&"Unknown".to_string()))).add_file(AttachmentType::from((img.as_slice(), "player_rank.png")))
        })
        .await?;

    Ok(())
}
