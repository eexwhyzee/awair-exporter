use reqwest;
use reqwest::Error;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use structopt::StructOpt;
use prometheus::{Encoder, GaugeVec, Opts, TextEncoder, Registry};
use hyper::{Body, Request, Response, Server};
use hyper::service::make_service_fn;
use hyper::service::service_fn;
use std::convert::Infallible;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up prometheus gauges
    let registry = Registry::new();
    let score = GaugeVec::new(
        Opts::new("score", "Current Awair Score")
        .namespace("awair")
        .subsystem("sensors"),
        &["airdata_url"]
    )?;
    registry.register(Box::new(score.clone()))?;

    let opts = Options::from_args();
    println!("listening at {}:{}", opts.address, opts.port);
    for url in &opts.airdata_urls {
        println!("Getting air data from {}", url);
        let d = get_air_data(url).await?;
        score.with_label_values(&[url]).set(d.score);
    }

    // Set up HTTP server to expose metrics
    let make_svc = make_service_fn(move |_| {
        let registry = registry.clone();
        async {
            Ok::<_, Infallible>(service_fn(move |_: Request<Body>| {
                let metric_families = registry.gather();
                let mut buffer = vec![];
                let encoder = TextEncoder::new();
                encoder.encode(&metric_families, &mut buffer).unwrap();

                let response = Response::builder()
                    .status(200)
                    .body(Body::from(buffer)).unwrap();

                async { Ok::<_, Infallible>(response) }
            }))
        }
    });

    let addr: std::net::SocketAddr = format!("{}:{}", opts.address, opts.port).parse()?;
    let server = Server::bind(&addr).serve(make_svc);

    println!("Serving metrics on http://{addr}/metrics");
    server.await.map_err(|e| e.into())
}

async fn get_air_data(airdata_url: &str) -> Result<AirData, Error> {
    let res: reqwest::Response = reqwest::get(airdata_url).await?;
    let data: AirData = res.json::<AirData>().await?;
    Ok(data)
}

fn generate_metrics() {
    todo!("generate prometheus metrics from air data")
}

