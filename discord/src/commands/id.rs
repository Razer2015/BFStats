use serenity::builder::CreateApplicationCommand;
use serenity::model;
use model::application::interaction::InteractionResponseType;
use model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    CommandDataOptionValue,
};
use serenity::prelude::Context;

pub async fn run(ctx: Context, command: &ApplicationCommandInteraction) {
    let option = command
        .data
        .options
        .get(0)
        .expect("Expected user option")
        .resolved
        .as_ref()
        .expect("Expected user object");

    let mut content;
    if let CommandDataOptionValue::User(user, _member) = option {
        content = format!("{}'s id is {}", user.tag(), user.id);
    } else {
        content = "Please provide a valid user".to_string();
    }

    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content(content))
        })
        .await
    {
        println!("Cannot respond to slash command: {}", why);
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("id").description("Get a user id").create_option(|option| {
        option
            .name("id")
            .description("The user to lookup")
            .kind(CommandOptionType::User)
            .required(true)
    })
}