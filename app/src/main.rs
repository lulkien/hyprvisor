mod opts;
use opts::Opts;

fn main() {
    let opts = Opts::from_env();

    println!("Action {:?}", opts);
}
