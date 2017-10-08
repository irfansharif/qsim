extern crate getopts;

use getopts::Options;
use std::env;

fn print_usage(program: &str, opts: &Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn parse_params(matches: &getopts::Matches) -> (i32, i32, i32, i32) {
    let a = match matches.opt_str("a") {
        Some(x) => x.parse::<i32>().unwrap(),
        None => 10,
    };
    let l = match matches.opt_str("l") {
        Some(x) => x.parse::<i32>().unwrap(),
        None => 2,
    };
    let c = match matches.opt_str("c") {
        Some(x) => x.parse::<i32>().unwrap(),
        None => 2,
    };
    let t = match matches.opt_str("t") {
        Some(x) => x.parse::<i32>().unwrap(),
        None => 20,
    };

    (a, l, c, t)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "help", "Display this message");
    opts.optopt(
        "a",
        "avg",
        &format!("Average number of generated packets/s (def: {})", 10),
        "NUM",
    );
    opts.optopt(
        "l",
        "len",
        &format!("Packet length; bits (def: {})", 2),
        "NUM",
    );
    opts.optopt(
        "c",
        "stime",
        &format!("Packet service time; bits/s (def: {})", 2),
        "NUM",
    );
    opts.optopt(
        "t",
        "ticks",
        &format!("Duration of simulation; TICKS (def: {})", 20),
        "NUM",
    );
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            println!("{}: illegal usage -- {}", program, f.to_string());
            std::process::exit(1)
        }
    };

    if matches.opt_present("h") {
        print_usage(&program, &opts);
        return;
    }

    let (_avg, _len, _stime, _ticks) = parse_params(&matches);
}
