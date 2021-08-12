mod global_data;
mod stats;
mod models;
mod images;
mod html_png;

use global_data::DatabasePool;
use handlebars::Handlebars;
use serde_json::Value;
use serenity::{client::Context, model::interactions::application_command::ApplicationCommand};

use stats::handle_top_interaction;

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
use sqlx::MySqlPool;
use crate::global_data::HandlebarsContext;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            match command.data.name.as_str() {
                "top" => {
                    if let Err(why) = handle_top_interaction(ctx, command).await {
                        println!("Error: {}", why)
                    };
                },
                "rank" => {
                    // if let Err(why) = handle_top_interaction(ctx, command).await {
                    //     println!("Error: {}", why)
                    // };
                },
                _ => (),
            };
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
