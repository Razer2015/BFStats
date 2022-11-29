use http::{header::USER_AGENT, StatusCode};

use crate::battlelog::models::{SearchResponse, IngameMetadataResponse};

use super::models::SearchResult;

pub async fn get_user(soldier_name: &str) -> Result<SearchResult, anyhow::Error> {
    let params = [("query", soldier_name.to_owned())];
    let client = reqwest::Client::new();
    let res = client
        .post("https://battlelog.battlefield.com/bf4/search/query/")
        .form(&params)
        .header(USER_AGENT, "BFStats")
        .send()
        .await?;

    let mut js = res.json::<SearchResponse>().await?;
    trace!("SearchResponse: {:#?}", js);

    for i in 0..js.data.len() {
        let result = &js.data[i];
        trace!("User: {:#?}", result);

        // Requires correct persona name. Apparently default parameters or overrides are not supported so not adding support for partial names now.
        if result.persona_name.ne(&soldier_name) {
            trace!("Not a correct persona");
            continue;
        }

        if result.namespace != "cem_ea_id" {
            trace!("Not a PC namespace");
            continue;
        }

        for val in result.games.values() {
            if val.parse::<i32>().unwrap() & 2048 == 0 {
                continue;
            }
            trace!("Has BF4");

            return Ok(js.data.remove(i));
        }
    }

    Err(anyhow::anyhow!("User not found"))
}

pub async fn search_user(soldier_name: &str) -> Result<SearchResult, anyhow::Error> {
    let params = [("query", soldier_name.to_owned())];
    let client = reqwest::Client::new();
    let res = client
        .post("https://battlelog.battlefield.com/bf4/search/query/")
        .form(&params)
        .header(USER_AGENT, "BFStats")
        .send()
        .await?;

    let js = res.json::<SearchResponse>().await?;
    trace!("SearchResponse: {:#?}", js);

    let mut valid_matches: Vec<SearchResult> = js.data
        .into_iter()
        .filter(|f| f.namespace.eq("cem_ea_id") && f.games.values().into_iter().any(|f| f.parse::<i32>().unwrap_or(0) & 2048 != 0))
        .collect();

    if valid_matches.len() == 1 {
        trace!("One match: {:#?}", valid_matches);

        return Ok(valid_matches.remove(0));
    }
    else if valid_matches.len() > 1 {
        trace!("Multiple matches: {:#?}", valid_matches);

        for i in 0..valid_matches.len() {
            let result = &valid_matches[i];
            trace!("User: {:#?}", result);

            if result.persona_name.eq_ignore_ascii_case(&soldier_name) {
                return Ok(valid_matches.remove(i));
            }
        }
    }

    Err(anyhow::anyhow!("User not found"))
}

pub async fn ingame_metadata(persona_id: u64) -> Result<IngameMetadataResponse, anyhow::Error> {
    let res = reqwest::Client::new()
        .get(format!(
            "https://battlelog.battlefield.com/api/bf4/pc/persona/1/{}/ingame_metadata",
            persona_id
        ))
        .header(USER_AGENT, "BFStats")
        .send()
        .await?;

    let status = res.status();

    let data_str = res.text().await?;
    trace!("{}", data_str);

    if status != StatusCode::OK {
        return Err(anyhow::anyhow!(data_str));
    }

    let data: IngameMetadataResponse = serde_json::from_str(&data_str)?;
    trace!("IngameMetadataResponse: {:#?}", data);

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_ingame_metadata() {
        let meta = ingame_metadata(806262072).await.unwrap();
        assert_eq!(806262072, meta.persona_id);
    }

    #[tokio::test]
    async fn get_user_test() {
        dbg!(get_user("xfileFIN").await.unwrap());
        // panic!()
    }

    #[tokio::test]
    async fn search_user_test() {
        dbg!(search_user("T3st1ngm").await.unwrap());
        // panic!()
    }
}
