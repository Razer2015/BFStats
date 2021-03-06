use handlebars::Handlebars;
use oxipng::PngError;
use serde::Serialize;

use crate::models::{ServerRankTemplate, ServerScoreTemplate, ServerTeamkillsTemplate};

pub async fn generate_server_ranks_image(handlebars: Handlebars<'_>, template_data: ServerScoreTemplate) -> Result<Vec<u8>, anyhow::Error> {
    generate_image(handlebars, "ServerRanks", &template_data, "table").await
}

pub async fn generate_server_teamkills_image(handlebars: Handlebars<'_>, template_data: ServerScoreTemplate) -> Result<Vec<u8>, anyhow::Error> {
    generate_image(handlebars, "ServerTeamkills", &template_data, "table").await
}

pub async fn generate_server_suicides_image(handlebars: Handlebars<'_>, template_data: ServerScoreTemplate) -> Result<Vec<u8>, anyhow::Error> {
    generate_image(handlebars, "ServerSuicides", &template_data, "table").await
}

pub async fn generate_server_teamkillsbyhour_image(handlebars: Handlebars<'_>, template_data: ServerTeamkillsTemplate) -> Result<Vec<u8>, anyhow::Error> {
    generate_image(handlebars, "ServerTeamkillsByHour", &template_data, "table").await
}

pub async fn generate_player_rank_image(handlebars: Handlebars<'_>, template_data: ServerRankTemplate) -> Result<Vec<u8>, anyhow::Error> {
    generate_image(handlebars, "PlayerRank", &template_data, "%23image").await
}

async fn generate_image<T>(handlebars: Handlebars<'_>, name: &str, template_data: T, element_to_render: &str) -> Result<Vec<u8>, anyhow::Error> 
where
        T: Serialize,
{
    let html = handlebars.render(name, &template_data)?;

    let client = reqwest::Client::new();
    let buffer = client.post(format!("{}api/html/render?element={}", dotenv::var("IMAGEAPI_URL").unwrap_or("http://localhost:3000/".to_string()), element_to_render))
        .body(html)
        .header("Content-Type", "text/plain")
        .send()
        .await?
        .bytes()
        .await?;

    Ok(compress_png(&buffer)?)
}

fn compress_png(buffer: &[u8]) -> Result<Vec<u8>, PngError> {
    let opts: oxipng::Options = Default::default();
    oxipng::optimize_from_memory(buffer, &opts)
}

// fn get_file_as_byte_vec(filename: &str) -> Vec<u8> {
//     let mut f = File::open(&filename).expect("no file found");
//     let metadata = fs::metadata(&filename).expect("unable to read metadata");
//     let mut buffer = vec![0; metadata.len() as usize];
//     f.read_exact(&mut buffer).expect("buffer overflow");

//     buffer
// }
