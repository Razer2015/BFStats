use std::{borrow::Cow, env, fs, path::Path};

use serenity::{client::Context, http::AttachmentType, model::interactions::{
        application_command::ApplicationCommandInteraction, InteractionResponseType,
    }};
use wkhtmltopdf::*;

use crate::{get_stats, global_data::{DatabasePool, HandlebarsContext}, images::generate_server_ranks_image, models::{Count, PlayerStats, ServerRankTemplate}};

pub async fn handle_top_interaction(ctx: Context, command: ApplicationCommandInteraction) {
    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response.kind(InteractionResponseType::DeferredChannelMessageWithSource)
        })
        .await
    {
        println!("Cannot respond to slash command: {}", why);
    }
    
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
        .await
        .unwrap()
        .count;

    let msg_id = match command
        .edit_original_interaction_response(&ctx.http, |response| {
            response.content(format!("Total players {}", total_players))
        })
        .await {
            Ok(msg) => msg.id.0,
            Err(_) => 0
        };

    // if let Err(why) = command
    //     .edit_original_interaction_response(&ctx.http, |response| {
    //         response.content(format!("Total players {}", total_players))
    //     })
    //     .await
    // {
    //     println!("Cannot respond to slash command: {}", why);
    // }

    let limit: i32 = 10;
    let offset: i32 = 0;

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
    .await
    .unwrap();

    println!("Data: {:#?}", data);

    let dir = env::current_dir().unwrap();
    let template_data = ServerRankTemplate {
        base_path: format!("file:///{}/templates/", dir.into_os_string().into_string().unwrap().replace('\\', "/")),
        players: get_stats(10, 0, pool).await.unwrap()
    };

    let img = generate_server_ranks_image(handlebars, template_data)
        .await
        .unwrap();

    // fs::write(Path::new("test_final.png"), &img).unwrap();

    if let Err(why) = command
        .edit_followup_message(&ctx.http, msg_id, |f| {
            f.content(format!("Total players {}", total_players)).add_file(AttachmentType::from((img.as_slice(), "top.png")))
        })
        .await
    {
        println!("Cannot respond to slash command: {}", why);
    }

    // if let Err(why) = command
    //     .edit_original_interaction_response(&ctx.http, |response| {
    //         response.create_embed(|e| {
    //             for (_pos, ps) in data.iter().enumerate() {
    //                 println!("{:#?}", ps);
    //                 e.field(ps.soldiername.as_ref().unwrap(), "aaa", false);
    //             }
    //             e.colour(0x00ff00)
    //         })
    //     })
    //     .await
    // {
    //     println!("Cannot respond to slash command: {}", why);
    // }

    // let data = sqlx::query!(
    //     "SELECT banner_user_id FROM permanent_bans WHERE guild_id = $1 AND user_id = $2",
    //     guild_id.0 as i64,
    //     member.user.id.0 as i64
    // )
    // .fetch_optional(&pool)
    // .await
    // .unwrap();
}
