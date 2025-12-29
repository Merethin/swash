mod models;
mod stream;

use std::{env, process::exit, error::Error,path::PathBuf};
use caramel::{ns::UserAgent, log::setup_log};
use reqwest::{ClientBuilder, Url};
use reqwest::header::{HeaderMap, HeaderValue};
use sqlx::PgPool;
use clap::{ArgAction, Command, arg, value_parser};
use log::{info, error};

const PROGRAM: & str = "swash";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: & str = "Merethin";

use crate::stream::{stream_data_dump_from_local, stream_data_dump_from_url};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    setup_log(vec![]);

    dotenv::dotenv().ok();

    let matches = Command::new("swash").arg(
        arg!(-k --keep "Don't clear the database tables before inserting new values").action(ArgAction::SetTrue)
    ).arg(
        arg!(
            -p --path [PATH] "Parse from nations.xml.gz and regions.xml.gz files in the target directory"
        ).required(false).value_parser(value_parser!(PathBuf))
    ).get_matches();

    let user_agent = UserAgent::read_from_env(PROGRAM, VERSION, AUTHOR);

    let mut headers = HeaderMap::new();

    headers.insert(
        "User-Agent", 
        HeaderValue::from_str(&user_agent.api()
    ).unwrap_or_else(|err| {
        error!("Invalid user agent: {} - {}", user_agent.api(), err);
        exit(1);
    }));

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
    } else {
        info!("Reading remote regions.xml.gz data dump");
        stream_data_dump_from_url(
            &pool, &client, Url::parse("https://www.nationstates.net/pages/regions.xml.gz")?
        ).await?;

        info!("Reading remote nations.xml.gz data dump");
        stream_data_dump_from_url(
            &pool, &client, Url::parse("https://www.nationstates.net/pages/nations.xml.gz")?
        ).await?;
    }

    info!("Finished");

    Ok(())
}
