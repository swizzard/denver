extern crate clap;

use clap::{App, Arg};
use denver::{merge_envs, run_with_env, set, split_line, Dir};

fn main() {
    let matches = App::new("denver")
        .version("0.1")
        .author("Sam Raker <sam.raker@gmail.com>")
        .about("dotenv-like")
        .arg(
            Arg::with_name("cmd")
                .short("c")
                .long("cmd")
                .value_name("CMD")
                .help("Command to run")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("env")
                .short("e")
                .long("env")
                .value_name("ENV")
                .help("Environment to use")
                .takes_value(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("set")
                .short("s")
                .long("set")
                .value_name("KEY=VALUE")
                .help("Set one-time vars")
                .takes_value(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("merge_left")
                .short("l")
                .long("merge_left")
                .help("Merge left, preserving earlier values")
                .takes_value(false),
        )
        .get_matches();
    let cmd = matches
        .value_of("cmd")
        .unwrap()
        .split_whitespace()
        .collect();
    let ens = matches.values_of("env").map_or(Vec::new(), |v| v.collect());
    let sets = matches.values_of("set").map_or(Vec::new(), |v| {
        v.map(|v| split_line(v.to_string())).collect()
    });
    let dir = {
        if matches.is_present("merge_left") {
            Dir::L
        } else {
            Dir::R
        }
    };
    let mut e = merge_envs(ens, dir).unwrap();
    for st in sets {
        if let Some((k, v)) = st {
            set(&mut e, k, v);
        }
    }
    run_with_env(cmd, e);
}
