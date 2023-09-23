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
}

fn main() {
    let opts = Opts::from_args();
    println!("listening at {}:{}", opts.address, opts.port);
}
