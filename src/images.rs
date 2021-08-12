use std::fs::{self, File};
use std::io::{Read};

use handlebars::Handlebars;
use oxipng::PngError;
use serde::Serialize;

use crate::html_png::html_to_png;
use crate::models::ServerRankTemplate;

pub async fn generate_server_ranks_image(handlebars: Handlebars<'_>, template_data: ServerRankTemplate) -> Result<Vec<u8>, anyhow::Error> {
    generate_image(handlebars, "ServerRanks", &template_data).await
}

async fn generate_image<T>(handlebars: Handlebars<'_>, name: &str, template_data: T) -> Result<Vec<u8>, anyhow::Error> 
where
        T: Serialize,
{
    let html = handlebars.render(name, &template_data)?;
    let buffer = html_to_png(html).await?;

    Ok(compress_png(&buffer)?)
}

fn compress_png(buffer: &Vec<u8>) -> Result<Vec<u8>, PngError> {
    let opts: oxipng::Options = Default::default();
    oxipng::optimize_from_memory(buffer, &opts)
}

fn get_file_as_byte_vec(filename: &String) -> Vec<u8> {
    let mut f = File::open(&filename).expect("no file found");
    let metadata = fs::metadata(&filename).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");

    buffer
}
