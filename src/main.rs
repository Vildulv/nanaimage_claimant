use anyhow::{ensure, Context, Result};
use reqwest::header::{HeaderValue, SET_COOKIE};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;

#[derive(Serialize, Deserialize, Debug)]
struct CookieStore {
    cookie: String,
}

const CONFIG_FILE: &str = "./cookie.json";
const CLAIM_API_URL: &str = "https://nanaimage.ai/api/claim-daily-credits";

fn main() -> Result<()> {
    let path = PathBuf::from(CONFIG_FILE);

    let store: CookieStore = CookieStore::load_from_file().context("Could not load cookie store")?;

    ensure!(!store.cookie.is_empty(), "Field cookie should not be empty");

    let client = reqwest::blocking::Client::new();
    let mut request_builder = client.post(CLAIM_API_URL);

    if !store.cookie.is_empty() {
        let cookie_val = HeaderValue::from_str(&store.cookie)?;

        request_builder = request_builder.header("Cookie", cookie_val);
    }

    let response = request_builder.send()?;

    // Check new cookie if present
    let new_cookie = response
        .headers()
        .get_all(SET_COOKIE)
        .iter()
        .filter_map(|val| val.to_str().ok())
        .find(|&val| val.starts_with("__Secure-authjs.session-token="))
        .map(|s| {
            s.split(';').next().unwrap_or(s).to_string()
        });

    // Updating cookie field
    if let Some(new_cookie_str) = new_cookie {
        let new_store = CookieStore { cookie: new_cookie_str };
        fs::write(&path, serde_json::to_string_pretty(&new_store)?)?;
        println!("Cookie updated and saved.");
    } else {
        println!("No new cookie found, leaving old one.");
    }

    let body = response.text()?;
    println!("Response body: {}", body);

    Ok(())
}

impl CookieStore {
    fn load_from_file() -> Result<Self> {
        let content = fs::read_to_string(CONFIG_FILE)
            .context("Failed to read cookie.json file")?;
        let store = serde_json::from_str(&content)
            .context("Failed to parse cookie.json")?;

        Ok(store)
    }
}


