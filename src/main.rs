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

fn main() {
    let opts = Opts::from_args();
    println!("listening at {}:{}", opts.address, opts.port);
    println!("Awair air-data URLS: {:?}", opts.airdata_urls);
}

fn getAirData() {
    todo!("get air data from awair local API")
}

fn generateMetrics() {
    todo!("generate prometheus metrics from air data")
}

