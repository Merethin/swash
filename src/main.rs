mod models;
mod stream;

use std::{env, process::exit, error::Error,path::PathBuf};
use caramel::{ns::UserAgent, log::setup_log};
use reqwest::{ClientBuilder, Url};
use reqwest::header::{HeaderMap, HeaderValue};
use sqlx::PgPool;
use clap::{ArgAction, Command, arg, value_parser};
use regex::Regex;
use log::{info, error};
use uuid::Uuid;

const PROGRAM: & str = "swash";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: & str = "Merethin";

use crate::stream::{stream_data_dump_from_local, stream_data_dump_from_url};

fn check_date(s: &str) -> Result<String, String> {
    let regex = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();

    if regex.is_match(s) {
        return Ok(s.to_string());
    } else {
        return Err("Invalid date format".into());
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    setup_log(vec![]);

    dotenv::dotenv().ok();

    let matches = Command::new("swash").arg(
        arg!(-k --keep "Don't clear the database tables before inserting new values").action(ArgAction::SetTrue)
    ).arg(
        arg!(
            -p --path [PATH] "Parse from nations.xml.gz and regions.xml.gz files in the target directory"
        ).required(false).value_parser(value_parser!(PathBuf)).conflicts_with("date")
    ).arg(
        arg!(
            -d --date [DATE] "Fetch a specific date instead of the latest data dumps, format: YYYY-MM-DD in PST time"
        ).required(false).value_parser(check_date).conflicts_with("path")
    ).get_matches();

    let user_agent = UserAgent::read_from_env(PROGRAM, VERSION, AUTHOR);

    let mut headers = HeaderMap::new();

    headers.insert(
        "User-Agent", 
        HeaderValue::from_str(&user_agent.api()).unwrap_or_else(|err| {
            error!("Invalid user agent: {} - {}", user_agent.api(), err);
            exit(1);
        }
    ));

    let client = ClientBuilder::new().default_headers(headers).build()?;

    let pool = PgPool::connect(&env::var("DATABASE_URL")?).await?;

    if !matches.get_flag("keep") {
        info!("Clearing existing tables");

        sqlx::query("TRUNCATE TABLE regions_dump").execute(&pool).await?;
        sqlx::query("TRUNCATE TABLE nations_dump").execute(&pool).await?;
    }

    if let Some(dump_path) = matches.get_one::<PathBuf>("path") {
        info!("Reading local regions.xml.gz data dump");
        stream_data_dump_from_local(
            &pool, dump_path.join("regions.xml.gz")
        ).await?;

        info!("Reading local nations.xml.gz data dump");
        stream_data_dump_from_local(
            &pool, dump_path.join("nations.xml.gz")
        ).await?;
    } else if let Some(date) = matches.get_one::<String>("date") {
        info!("Reading remote regions.xml.gz data dump");
        stream_data_dump_from_url(
            &pool, &client, Url::parse(&format!("https://www.nationstates.net/archive/regions/{}-regions-xml.gz", date))?
        ).await?;

        info!("Reading remote nations.xml.gz data dump");
        stream_data_dump_from_url(
            &pool, &client, Url::parse(&format!("https://www.nationstates.net/archive/nations/{}-nations-xml.gz", date))?
        ).await?;
    } else {
        let uuid = Uuid::new_v4();

        info!("Reading remote regions.xml.gz data dump");
        stream_data_dump_from_url(
            &pool, &client, Url::parse(&format!("https://www.nationstates.net/pages/regions.xml.gz?v={}", uuid))?
        ).await?;

        info!("Reading remote nations.xml.gz data dump");
        stream_data_dump_from_url(
            &pool, &client, Url::parse(&format!("https://www.nationstates.net/pages/nations.xml.gz?v={}", uuid))?
        ).await?;
    }

    info!("Finished");

    Ok(())
}
