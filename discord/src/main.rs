mod global_data;
mod images;
mod models;
mod stats;
mod vip;
mod battlelog;
mod commands;

use global_data::DatabasePool;
use handlebars::Handlebars;
use serde_json::Value;
use serenity::{client::Context, model::interactions::application_command::ApplicationCommand};

use stats::{handle_rank_interaction, handle_top_interaction};
use vip::{handle_vip_interaction};

use dotenv::dotenv;
use serde_json::json;

use crate::{global_data::HandlebarsContext, models::Server, stats::{
        handle_teamkillsbyhour_interaction, handle_top_suicides_interaction,
        handle_top_teamkills_interaction,
    }};
use serenity::{
    async_trait,
    http::Http,
    model::{gateway::Ready, interactions::Interaction},
    prelude::*,
};
use sqlx::MySqlPool;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            match command.data.name.as_str() {
                "id" => {
                    commands::id::run(ctx, &command);
                },
                "top" => {
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
                            if let Err(why) = handle_top_interaction(ctx, command).await {
                                println!("Error: {}", why)
                            };
                        }
                        "type_teamkills" => {
                            if let Err(why) = handle_top_teamkills_interaction(ctx, command).await {
                                println!("Error: {}", why)
                            };
                        }
                        "type_teamkillbyhour" => {
                            if let Err(why) = handle_teamkillsbyhour_interaction(ctx, command).await
                            {
                                println!("Error: {}", why)
                            };
                        }
                        "type_suicides" => {
                            if let Err(why) = handle_top_suicides_interaction(ctx, command).await {
                                println!("Error: {}", why)
                            };
                        }
                        _ => println!("Unknown score type: {}", score_type),
                    };
                }
                "rank" => {
                    if let Err(why) = handle_rank_interaction(ctx, command).await {
                        println!("Error: {}", why)
                    };
                }
                "vip" => {
                    if let Err(why) = handle_vip_interaction(ctx, command).await {
                        println!("Error: {}", why)
                    };
                }
                _ => (),
            };
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        // Fetch currently registered guild commands
        let commands = get_guild_commands(ctx.clone()).await.unwrap();
        println!("Registered GUILD commands ({})", commands.len());
        for command in &commands {
            println!("Command {} with id {}", command.name, command.id);
        }
        println!();
        //delete_guild_command(ctx.clone(), 956964456947150858).await;

        // Fetch currently registered global commands
        let commands = get_global_commands(ctx.clone()).await.unwrap();
        println!("Registered GLOBAL commands ({})", commands.len());
        for command in &commands {
            println!("Command {} with id {}", command.name, command.id);
        }
        println!();
        //delete_global_command(ctx.clone(), 956964456947150858).await;

        if let Err(err) = add_guild_command(
            ctx.clone(),
            &json!({
                "name": "top",
                "description": "Command to get top players in the server.",
                "options": [
                    {
                        "name": "type",
                        "description": "The type of stats",
                        "type": 3,
                        "required": true,
                        "choices": [
                            {
                                "name": "Score",
                                "value": "type_score"
                            },
                            {
                                "name": "Teamkills",
                                "value": "type_teamkills"
                            },
                            {
                                "name": "TKH (players with less than 1 day playtime excluded)",
                                "value": "type_teamkillbyhour"
                            },
                            {
                                "name": "Suicides",
                                "value": "type_suicides"
                            }
                        ]
                    },
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
            }),
        )
        .await
        {
            println!("Error adding guild command: {}", err)
        }

        // Rank command
        let pool = {
            let data_read = &ctx.data.read().await;
            data_read.get::<DatabasePool>().unwrap().clone()
        };

        let servers = sqlx::query_as!(Server, "SELECT serverid as server_id, servername as server_name FROM tbl_server")
            .fetch_all(&pool)
            .await;

        if servers.is_ok() && servers.as_ref().unwrap().len() > 1 { // Multiple servers
            
            let choices = servers.unwrap().iter().map(|server| json!( {
                "name": no_space(server.server_name.as_ref().unwrap_or(&"Unknown".to_string()).to_string()).trim(),
                "value": server.server_id
            }))
            .collect::<Vec<Value>>();

            if let Err(err) = add_guild_command(
                ctx.clone(),
                &json!({
                    "name": "rank",
                    "description": "Command to get your rank in the server.",
                    "options": [
                        {
                            "name": "server",
                            "description": "Which server you want the stats from?",
                            "type": 4,
                            "required": true,
                            "choices": choices
                        },
                        {
                            "type": 3,
                            "name": "soldiername",
                            "description": "Name of the soldier (can be partial but must have only one match).",
                            "required": true
                        }
                    ]
                }),
            )
            .await
            {
                println!("Error adding guild command: {}", err)
            }
        }
        else { // One server
            if let Err(err) = add_guild_command(
                ctx.clone(),
                &json!({
                    "name": "rank",
                    "description": "Command to get your rank in the server.",
                    "options": [
                        {
                            "type": 3,
                            "name": "soldiername",
                            "description": "Name of the soldier (can be partial but must have only one match).",
                            "required": true
                        }
                    ]
                }),
            )
            .await
            {
                println!("Error adding guild command: {}", err)
            }
        }

        // VIP command
        if let Err(err) = add_global_command(
            ctx.clone(),
            &json!({
                "name": "vip",
                "description": "Command to get your VIP information in the server.",
                "options": [ ]
            }),
        )
        .await
        {
            println!("Error adding guild command: {}", err)
        }
    }
}

fn no_space(x : String) -> String{
    x.replace(' ', "-")
}

#[allow(dead_code)]
async fn add_guild_command(
    ctx: Context,
    command_json: &Value,
) -> Result<ApplicationCommand, SerenityError> {
    let guild_id: u64 = dotenv::var("GUILD_ID")
        .expect("Expected an guild id in the environment")
        .parse()
        .expect("guild id is not a valid id");

    Http::create_guild_application_command(&ctx.http, guild_id, command_json).await
}

#[allow(dead_code)]
async fn add_global_command(
    ctx: Context,
    command_json: &Value,
) -> Result<ApplicationCommand, SerenityError> {
    Http::create_global_application_command(&ctx.http, command_json).await
}

#[allow(dead_code)]
async fn get_guild_commands(
    ctx: Context,
) -> Result<Vec<ApplicationCommand>, SerenityError> {
    let guild_id: u64 = dotenv::var("GUILD_ID")
        .expect("Expected an guild id in the environment")
        .parse()
        .expect("guild id is not a valid id");

    Http::get_guild_application_commands(&ctx.http, guild_id).await
}

#[allow(dead_code)]
async fn get_global_commands(
    ctx: Context,
) -> Result<Vec<ApplicationCommand>, SerenityError> {
    Http::get_global_application_commands(&ctx.http).await
}

#[allow(dead_code)]
async fn delete_guild_command(
    ctx: Context,
    command_id: u64
) -> Result<(), SerenityError> {
    let guild_id: u64 = dotenv::var("GUILD_ID")
        .expect("Expected an guild id in the environment")
        .parse()
        .expect("guild id is not a valid id");

    Http::delete_guild_application_command(&ctx.http, guild_id, command_id).await
}

#[allow(dead_code)]
async fn delete_global_command(
    ctx: Context,
    command_id: u64
) -> Result<(), SerenityError> {
    Http::delete_global_application_command(&ctx.http, command_id).await
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

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = dotenv::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let db_url = dotenv::var("DATABASE_URL")
        .expect("Expected a database connection string in the environment");

    // The Application Id is usually the Bot User Id.
    let application_id: u64 = dotenv::var("APPLICATION_ID")
        .expect("Expected an application id in the environment")
        .parse()
        .expect("application id is not a valid id");

    // Build our client.
    let mut client = Client::builder(token, GatewayIntents::empty())
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
        handlebars
            .register_template_file("ServerTeamkills", "./templates/ServerTeamkills.html")
            .unwrap();
        handlebars
            .register_template_file("ServerSuicides", "./templates/ServerSuicides.html")
            .unwrap();
        handlebars
            .register_template_file("PlayerRank", "./templates/PlayerRank.html")
            .unwrap();
        handlebars
            .register_template_file(
                "ServerTeamkillsByHour",
                "./templates/ServerTeamkillsByHour.html",
            )
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
