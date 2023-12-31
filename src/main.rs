use chrono::{DateTime, Utc};
use env_logger::Env;
use lazy_static::lazy_static;
use log::{info, warn};
use prometheus::{Encoder, GaugeVec, Opts, TextEncoder};
use reqwest;
use reqwest::Error;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use warp::{http::Response, Filter};

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Awair Local API Prometheus Exporter",
    about = "A CLI tool to export sensor data from the Awair Local API to Prometheus"
)]
struct Options {
    #[structopt(
        long,
        short,
        default_value = "0.0.0.0",
        help = "Metrics server address"
    )]
    address: String,

    #[structopt(long, short, default_value = "8000", help = "Metrics server port")]
    port: u16,

    #[structopt(
        long,
        short = "u",
        required = true,
        takes_value = true,
        min_values = 1,
        help = "List of air-data URLs exposed from the Awair Local API"
    )]
    airdata_urls: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AirData {
    timestamp: DateTime<Utc>,
    score: f64,
    temp: f64,
    humid: f64,
    co2: f64,
    voc: f64,
    pm25: f64,
}

impl AirData {
    async fn new(airdata_url: &str) -> Result<AirData, Error> {
        let res: reqwest::Response = reqwest::get(airdata_url).await?;
        let data: AirData = res.json::<AirData>().await?;
        Ok(data)
    }
}
// Declare and initialize global metrics
lazy_static! {
    pub static ref SCORE_GAUGE: GaugeVec = {
        let gauge = GaugeVec::new(
            Opts::new("score", "Current Awair Score")
                .namespace("awair")
                .subsystem("sensors"),
            &["airdata_url"],
        )
        .unwrap();
        prometheus::register(Box::new(gauge.clone())).unwrap();
        gauge
    };
    pub static ref TEMP_GAUGE: GaugeVec = {
        let gauge = GaugeVec::new(
            Opts::new("temp", "Current temperature in celcius")
                .namespace("awair")
                .subsystem("sensors"),
            &["airdata_url"],
        )
        .unwrap();
        prometheus::register(Box::new(gauge.clone())).unwrap();
        gauge
    };
    pub static ref HUMIDITY_GAUGE: GaugeVec = {
        let gauge = GaugeVec::new(
            Opts::new("humidity", "Current relative humidity")
                .namespace("awair")
                .subsystem("sensors"),
            &["airdata_url"],
        )
        .unwrap();
        prometheus::register(Box::new(gauge.clone())).unwrap();
        gauge
    };
    pub static ref CO2_GAUGE: GaugeVec = {
        let gauge = GaugeVec::new(
            Opts::new("co2", "Current CO2 measurement in parts per million")
                .namespace("awair")
                .subsystem("sensors"),
            &["airdata_url"],
        )
        .unwrap();
        prometheus::register(Box::new(gauge.clone())).unwrap();
        gauge
    };
    pub static ref VOC_GAUGE: GaugeVec = {
        let gauge = GaugeVec::new(
            Opts::new(
                "voc",
                "Current Volatile Organic Compound measurement in parts per billion",
            )
            .namespace("awair")
            .subsystem("sensors"),
            &["airdata_url"],
        )
        .unwrap();
        prometheus::register(Box::new(gauge.clone())).unwrap();
        gauge
    };
    pub static ref PM25_GAUGE: GaugeVec = {
        let gauge = GaugeVec::new(
            Opts::new(
                "pm25",
                "Current concentration of 2.5 micron particles in micrograms per meter cubed",
            )
            .namespace("awair")
            .subsystem("sensors"),
            &["airdata_url"],
        )
        .unwrap();
        prometheus::register(Box::new(gauge.clone())).unwrap();
        gauge
    };
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    // Get options from command line args
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
    info!("Serving metrics on http://{addr}/metrics");
    let (_addr, fut) = warp::serve(metrics_route).bind_with_graceful_shutdown(addr, async move {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen to shutdown signal");
    });
    fut.await;
    Ok(())
}

async fn generate_metrics(airdata_urls: Vec<String>) {
    loop {
        for url in &airdata_urls {
            let d = AirData::new(url).await;
            match d {
                Ok(value) => {
                    SCORE_GAUGE.with_label_values(&[url]).set(value.score);
                    TEMP_GAUGE.with_label_values(&[url]).set(value.temp);
                    HUMIDITY_GAUGE.with_label_values(&[url]).set(value.humid);
                    CO2_GAUGE.with_label_values(&[url]).set(value.co2);
                    VOC_GAUGE.with_label_values(&[url]).set(value.voc);
                    PM25_GAUGE.with_label_values(&[url]).set(value.pm25);
                }
                Err(e) => {
                    warn!("unable to get air-data from {url}: {e}");
                }
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
    }
}
