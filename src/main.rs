use reqwest;
use reqwest::Error;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Awair Local API Prometheus Exporter",
    about = "A CLI tool to export sensor data from the Awair Local API to Prometheus"
)]
struct Opts {
    #[structopt(long, short,
        help = "Listen address")]
    address: String,

    #[structopt(long, short, 
        help = "Listen port")]
    port: u16,

    #[structopt(long, short = "u", required = true, takes_value = true, min_values = 1,
        help = "List of air-data URLs exposed from the Awair Local API"
    )]
    airdata_urls: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AirData {
    timestamp: DateTime<Utc>,
    score: u8,
    dew_point: f64,
    temp: f64,
    humid: f64,
    abs_humid: f64,
    co2: u32,
    co2_est: u32,
    co2_est_baseline: u32,
    voc: u32,
    voc_baseline: u32,
    voc_h2_raw: u32,
    voc_ethanol_raw: u32,
    pm25: u32,
    pm10_est: u32,
}

#[tokio::main]
async fn main() {
    let opts = Opts::from_args();
    println!("listening at {}:{}", opts.address, opts.port);
    for url in &opts.airdata_urls {
        println!("Getting air data from {}", url);
        let d = get_air_data(url).await;
        println!("{:?}", d.unwrap());

    }
}

async fn get_air_data(airdata_url: &str) -> Result<AirData, Error> {
    let res: reqwest::Response = reqwest::get(airdata_url).await?;
    let data: AirData = res.json::<AirData>().await?;
    Ok(data)
}

fn generate_metrics() {
    todo!("generate prometheus metrics from air data")
}

