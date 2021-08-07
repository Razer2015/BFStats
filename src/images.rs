use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path};

use handlebars::Handlebars;
use oxipng::PngError;
use rand::Rng;
use serde::Serialize;
use wkhtmltopdf::{ImageApplication, ImageFormat};

use crate::models::ServerRankTemplate;

pub fn generate_server_ranks_image(handlebars: Handlebars, template_data: ServerRankTemplate) -> Result<Vec<u8>, anyhow::Error> {
    match save_file_from_template(handlebars, "ServerRanks", &template_data) {
        Ok(path) => {
            Ok(generate_image(path)?)
        },
        Err(err) => panic!("Error: {}", err),
    }
}

fn save_file_from_template<T>(handlebars: Handlebars, name: &str, template_data: T) -> Result<String, anyhow::Error>
where
        T: Serialize,
{
    let data = handlebars.render(name, &template_data)?;

    let mut rng = rand::thread_rng();
    fs::create_dir_all("tempfiles")?;
    let temp_path = format!("tempfiles\\{}.html", rng.gen::<u32>());
    let path = Path::new(&temp_path);
    let display = path.display();

    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, why),
        Ok(file) => file,
    };

    match file.write_all(data.as_bytes()) {
        Err(why) => panic!("couldn't write to {}: {}", display, why),
        Ok(_) => println!("successfully wrote to {}", display),
    }

    Ok(temp_path)
}

fn generate_image(temp_path: String) -> Result<Vec<u8>, anyhow::Error> {
    // TODO: This can't be initialized twice
    let image_app = ImageApplication::new()
        .expect("Failed to init image application");

    let mut buffer = Vec::new();
    unsafe {
        let mut imageout = image_app
            .builder()
            .format(ImageFormat::Png)
            .global_setting("loadPage.blockLocalFileAccess", "false")
            .build_from_path(&temp_path)
            .expect("failed to build image");
        
        if let Err(error) = imageout.read_to_end(&mut buffer) {
            println!("Error: {}", error)
        }
    }

    let path = Path::new(&temp_path);
    let display = path.display();
    match fs::remove_file(&path) {
        Err(why) => panic!("couldn't delete {}: {}", display, why),
        Ok(_) => println!("successfully deleted {}", display),
    };

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
