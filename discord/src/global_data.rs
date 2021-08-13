use handlebars::Handlebars;
use serenity::prelude::TypeMapKey;
use sqlx::MySqlPool;

// Example from https://gitlab.com/vicky5124/robo-arc/-/blob/master/src/global_data.rs

pub struct DatabasePool; // A pool of connections to the database.
pub struct HandlebarsContext; // A context for handlebars

impl TypeMapKey for DatabasePool {
    type Value = MySqlPool;
}

impl TypeMapKey for HandlebarsContext {
    type Value = Handlebars<'static>;
}
