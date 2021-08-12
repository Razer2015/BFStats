//! Converting HTML to images using wkhtmltopdf.

use std::{convert::Infallible, fs::{self, File}, io::{Read, Write}, path::{Path, PathBuf}, sync::Arc};
use rand::Rng;
use unasync::{UnAsync, UnSync};
use wkhtmltopdf::{ImageApplication, ImageFormat};
use tokio::task::spawn_blocking;

lazy_static::lazy_static! {
    static ref HTML_PNG: UnAsync<Html2Png> = UnAsync::new_infallible();
}

pub(crate) struct Html2Png(Result<ImageApplication, Arc<wkhtmltopdf::Error>>);

impl UnSync for Html2Png {
    type E = Infallible;
    type Request = PathBuf; // path to tempfile with html
    type Response = Result<Vec<u8>, Arc<wkhtmltopdf::Error>>; // generated image

    fn create() -> Result<Self, Self::E> where Self: Sized {
        // same as: Ok(Self(ImageApplication::new().map_err(|err: wkhtmltopdf::Error| Arc::new(err))))
        Ok(Self(ImageApplication::new().map_err(Arc::new)))
    }

    fn process(&mut self, path: Self::Request) -> Self::Response {
        match &self.0 {
            Ok(imgapp) => {
                let mut image = unsafe {
                    imgapp.builder()
                        .format(ImageFormat::Png)
                        .global_setting("loadPage.blockLocalFileAccess", "false")
                        .build_from_path(&path)?
                };

                let mut buffer = Vec::new();
                image.read_to_end(&mut buffer);

                match fs::remove_file(&path) {
                    Err(why) => panic!("couldn't delete {}: {}", path.display(), why),
                    Ok(_) => println!("successfully deleted {}", path.display()),
                };

                Ok(buffer)
            },
            Err(wk_err) => Err(wk_err.clone()),
        }
    }
}

pub async fn html_to_png(html: String) -> anyhow::Result<Vec<u8>> {
    let mut rng = rand::thread_rng();
    fs::create_dir_all("tempfiles")?;
    let temp_path = format!("tempfiles\\{}.html", rng.gen::<u32>());
    let path = Path::new(&temp_path);
    let mut file = File::create(&path)?;
    let file = spawn_blocking(move || {
        file.write(html.as_bytes());
        file
    }).await?;
    
    Ok(HTML_PNG.query(path.to_owned()).await?)
}