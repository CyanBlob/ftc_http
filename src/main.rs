use ftc_http::*;
use structopt::StructOpt;

fn main() {
    let opt = Ftc::from_args();
    let conf = AppConfig::default();
    run(opt);
}