use reqwest;
use reqwest::Error;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use structopt::StructOpt;
use prometheus::{Encoder, GaugeVec, Opts, TextEncoder};
use lazy_static::lazy_static;
use warp::{Filter, http::Response};


#[derive(Debug, StructOpt)]
#[structopt(
    name = "Awair Local API Prometheus Exporter",
    about = "A CLI tool to export sensor data from the Awair Local API to Prometheus"
)]
struct Options {
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
    score: f64,
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

// Declare and initialize global metrics
lazy_static! {
    pub static ref SCORE_GAUGE: GaugeVec = {
        let gauge = GaugeVec::new(
            Opts::new("score", "Current Awair Score")
            .namespace("awair")
            .subsystem("sensors"),
            &["airdata_url"]
        ).unwrap();
        prometheus::register(Box::new(gauge.clone())).unwrap();
        gauge
    };
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Options::from_args();

    // Generate metrics in the background
    tokio::spawn(generate_metrics(opts.airdata_urls));

    // Set up endpoint to expose metrics
    let metrics_route = warp::path!("metrics").map(|| {
        let encoder = TextEncoder::new();
        let metric_families = prometheus::gather();
        let mut buffer = vec![];
        encoder.encode(&metric_families, &mut buffer).unwrap();
        let response = Response::builder()
            .header("content-type", encoder.format_type())
            .body(buffer)
            .unwrap();

        warp::reply::with_status(response, warp::http::StatusCode::OK)
    });

    // Start HTTP server
    let addr: std::net::SocketAddr = format!("{}:{}", opts.address, opts.port).parse()?;
    println!("Serving metrics on http://{addr}/metrics");
    warp::serve(metrics_route)
        .run(addr)
        .await;
    Ok(())
}

async fn get_air_data(airdata_url: &str) -> Result<AirData, Error> {
    let res: reqwest::Response = reqwest::get(airdata_url).await?;
    let data: AirData = res.json::<AirData>().await?;
    Ok(data)
}

async fn generate_metrics(airdata_urls: Vec<String>) {
    loop {
        for url in &airdata_urls {
            let d = get_air_data(url).await.unwrap();
            SCORE_GAUGE.with_label_values(&[url]).set(d.score);
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
    }
}

