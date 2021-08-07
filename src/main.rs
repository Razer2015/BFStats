mod global_data;
mod stats;
mod models;
mod images;

use std::{env, fs::{self, File}, path::{Path, PathBuf}};
use oxipng::Options;
use rand::prelude::*;

use global_data::DatabasePool;
use handlebars::{Handlebars, Helper, Output, RenderContext, RenderError};
use models::ServerRankTemplate;
use serde_json::Value;
use serenity::{client::Context, model::interactions::application_command::ApplicationCommand};

use stats::handle_top_interaction;
use tempfile::NamedTempFile;
use std::io::{self, Write};

use serde_json::json;
use dotenv::dotenv;

use serenity::{
    async_trait,
    http::Http,
    model::{
        gateway::Ready,
        interactions::Interaction
    },
    prelude::*,
};
use sqlx::{MySql, MySqlPool, Pool};
use wkhtmltopdf::{ImageApplication, ImageFormat};

use crate::{global_data::HandlebarsContext, images::generate_server_ranks_image, models::PlayerStats};


struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        

        // let data = sqlx::query!(
        //     "SELECT banner_user_id FROM permanent_bans WHERE guild_id = $1 AND user_id = $2",
        //     guild_id.0 as i64,
        //     member.user.id.0 as i64
        // )
        // .fetch_optional(&pool)
        // .await
        // .unwrap();

        if let Interaction::ApplicationCommand(command) = interaction {
            match command.data.name.as_str() {
                "top" => {
                    handle_top_interaction(ctx, command).await;
                },
                _ => (),
            };

            // if let Err(why) = command
            //     .create_interaction_response(&ctx.http, |response| {
            //         response
            //             .kind(InteractionResponseType::ChannelMessageWithSource)
            //             .interaction_response_data(|message| message.content(content))
            //     })
            //     .await
            // {
            //     println!("Cannot respond to slash command: {}", why);
            // }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        if let Err(err) = add_guild_command(ctx, &json!({
                    "name": "top",
                    "description": "Command to get top players in the server.",
                    "options": [
                        {
                            "type": 4,
                            "name": "count",
                            "description": "How many to show? Default 10 and max 20.",
                            "required": false
                        },
                        {
                            "type": 4,
                            "name": "offset",
                            "description": "From which offset to show? Default 0.",
                            "required": false
                        }
                    ]
                })).await {
            println!("Error adding guild command: {}", err)
        }
    }
}

async fn add_guild_command(ctx: Context, command_json: &Value) -> Result<ApplicationCommand, SerenityError> {
    let guild_id: u64 = dotenv::var("GUILD_ID")
        .expect("Expected an guild id in the environment")
        .parse()
        .expect("guild id is not a valid id");

    Http::create_guild_application_command(&ctx.http, guild_id, command_json).await
}

// async fn add_guild_command_with_permissions(ctx: Context, command_json: &Value, permissions_json: &Value) {
//     let guild_id: u64 = dotenv::var("GUILD_ID")
//         .expect("Expected an guild id in the environment")
//         .parse()
//         .expect("guild id is not a valid id");

//     if let Ok(cmd) = add_guild_command(ctx.clone(), &command_json).await {           
//         let _ = Http::edit_guild_application_command_permissions(&ctx.http, guild_id, cmd.id.0, permissions_json).await;
//     }
// }

async fn get_stats(limit: i32, offset: i32, pool: Pool<MySql>) -> Result<Vec<PlayerStats>, sqlx::Error> {
    sqlx::query_as!(
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
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = dotenv::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let db_url = dotenv::var("DATABASE_URL").expect("Expected a database connection string in the environment");

    // The Application Id is usually the Bot User Id.
    let application_id: u64 = dotenv::var("APPLICATION_ID")
        .expect("Expected an application id in the environment")
        .parse()
        .expect("application id is not a valid id");


    // let pool = MySqlPool::connect(&db_url).await.unwrap();
    // let dir = env::current_dir().unwrap();
    // let template_data = ServerRankTemplate {
    //     base_path: format!("file:///{}/templates/", dir.into_os_string().into_string().unwrap().replace('\\', "/")),
    //     players: get_stats(10, 0, pool).await.unwrap()
    // };

    // let mut handlebars = Handlebars::new();
    // handlebars
    //     .register_template_file("ServerRanks", "./templates/ServerRanks.html")
    //     .unwrap();

    // let img = generate_server_ranks_image(handlebars, template_data)
    //     .unwrap();

    // fs::write(Path::new("test_final.png"), &img).unwrap();

    
    let image_app = ImageApplication::new()
        .expect("Failed to init image application");


    // Build our client.
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .application_id(application_id)
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;

        let pool = MySqlPool::connect(&db_url).await.unwrap();
        data.insert::<DatabasePool>(pool);

        let mut handlebars = Handlebars::new();
        handlebars
            .register_template_file("ServerRanks", "./templates/ServerRanks.html")
            .unwrap();

        data.insert::<HandlebarsContext>(handlebars);
    }

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
