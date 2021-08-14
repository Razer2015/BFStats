
use http::{header::USER_AGENT};

use crate::models::{SearchResponse, SearchResult};

pub async fn search_user(soldier_name: &str) -> Result<SearchResult, anyhow::Error> {
    let params = [("query", soldier_name.to_owned())];
    let client = reqwest::Client::new();
    let res = client
        .post("https://battlelog.battlefield.com/bf4/search/query/")
        .form(&params)
        .header(USER_AGENT, "BFStats")
        .send()
        .await?;

    let mut js = res.json::<SearchResponse>().await?;
    //println!("SearchResponse: {:#?}", js);

    for i in 0..js.data.len() {
        let result = &js.data[i];
        //println!("User: {:#?}", result);

        // Requires correct persona name. Apparently default parameters or overrides are not supported so not adding support for partial names now.
        if result.persona_name.ne(&soldier_name) {
            //println!("Not a correct persona");
            continue;
        }

        if result.namespace != "cem_ea_id" {
            //println!("Not a PC namespace");
            continue;
        }

        for val in result.games.values() {
            if val.parse::<i32>().unwrap() & 2048 == 0 {
                continue;
            }
            //println!("Has BF4");

            return Ok(js.data.remove(i))
        }
    }

    Err(anyhow::anyhow!("User not found"))
}