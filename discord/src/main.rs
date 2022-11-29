#[macro_use]
extern crate log;

mod global_data;
mod images;
mod models;
mod stats;
mod vip;
mod commands;
mod logging;
mod battlelog;

use global_data::DatabasePool;
use handlebars::Handlebars;
use serenity::{client::Context, model::{prelude::{GuildId, interaction::Interaction, command::Command/*, CommandId */}}};

use dotenv::dotenv;

use crate::{global_data::HandlebarsContext, models::Server};
use serenity::{
    async_trait,
    model::{gateway::Ready},
    prelude::*,
};
use sqlx::MySqlPool;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let result = match command.data.name.as_str() {
                "id" => {
                    commands::id::run(ctx, &command).await
                },
                "top" => {
                    commands::top::run(ctx, &command).await
                }
                "rank" => {
                    commands::rank::run(ctx, &command).await
                }
                "vip" => {
                    commands::vip::run(ctx, &command).await
                }
                "search" => {
                    commands::search::run(ctx, &command).await
                }
                _ => Ok(()),
            };

            if let Err(why) = result
            {
                error!("Prosessing slash command failed: {}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        let guild_id = GuildId(
            dotenv::var("GUILD_ID")
                .expect("Expected GUILD_ID in environment")
                .parse()
                .expect("GUILD_ID must be an integer"),
        );

        // // Fetch currently registered guild commands
        // let guild_commands = GuildId::get_application_commands(&guild_id, &ctx.http).await.unwrap();
        // info!("Registered GUILD commands ({})", guild_commands.len());
        // for command in &guild_commands {
        //     info!("Command {} with id {}", command.name, command.id);
        // }
        // info!("");
        // //GuildId::delete_application_command(&guild_id, &ctx.http, CommandId(956964456947150858)).await.unwrap();

        // // Fetch currently registered global commands
        // let global_commands = Command::get_global_application_commands(&ctx.http).await.unwrap();
        // info!("Registered GLOBAL commands ({})", global_commands.len());
        // for command in &global_commands {
        //     info!("Command {} with id {}", command.name, command.id);
        // }
        // info!("");
        // //Command::delete_global_application_command(&ctx.http, CommandId(956964456947150858)).await.unwrap();

        let pool = {
            let data_read = &ctx.data.read().await;
            data_read.get::<DatabasePool>().unwrap().clone()
        };
    
        let servers = sqlx::query_as!(Server, "SELECT serverid as server_id, servername as server_name FROM tbl_server")
            .fetch_all(&pool)
            .await
            .unwrap_or(Vec::new());

        // Register guild commands (new style)
        let guild_commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands
                .create_application_command(|command| commands::id::register(command))
                .create_application_command(|command| commands::top::register(command))
                .create_application_command(|command| commands::rank::register(command, servers))
                .create_application_command(|command| commands::search::register(command))
        })
        .await
        .unwrap();

        trace!("I now have the following guild slash commands: {:#?}", guild_commands);
        info!("Registered GUILD commands ({})", guild_commands.len());
        for command in &guild_commands {
            info!("Command {} with id {}", command.name, command.id);
        }
        info!("");

        // Register global commands (new style)
        let global_command = Command::create_global_application_command(&ctx.http, |command| {
            commands::vip::register(command)
        })
        .await
        .unwrap();

        trace!("I created the following global slash commands: {:#?}", global_command);
        info!("Registered GLOBAL command {} with id {}", global_command.name, global_command.id);
        info!("");
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    logging::init_logging();

    info!("BFStats starting");

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
        handlebars.register_script_helper_file("to_int", "./templates/script_helpers/to_int.rhai").unwrap();
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
        error!("Client error: {:?}", why);
    }
}
