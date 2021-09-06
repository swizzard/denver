use crate::{merge_envs, set, split_line, Dir};
use clap::{App, Arg, ArgMatches};
use std::collections::HashMap;
use std::io;

pub fn denver_arg<'a, 'b>() -> App<'a, 'b> {
    App::new("denver")
        .version("0.1")
        .author("Sam Raker <sam.raker@gmail.com>")
        .about("dotenv-like")
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
        .arg(
            Arg::with_name("cmd")
                .short("c")
                .long("cmd")
                .value_name("CMD")
                .help("Command to run")
                .takes_value(true)
                .required(true)
                .index(1),
        )
}

pub fn get_args<'a>(
    matches: &'a ArgMatches,
) -> io::Result<(Vec<&'a str>, HashMap<String, String>)> {
    let cmd = get_cmd(matches);
    let ens = get_ens(matches);
    let sets = get_sets(matches);
    let dir = get_dir(matches);
    let mut e = merge_envs(ens, dir)?;
    for st in sets {
        if let Some((k, v)) = st {
            set(&mut e, k, v);
        }
    }
    Ok((cmd, e))
}

fn get_cmd<'a>(matches: &'a ArgMatches<'a>) -> Vec<&'a str> {
    matches
        .value_of("cmd")
        .unwrap()
        .split_whitespace()
        .collect()
}

fn get_ens<'a>(matches: &'a ArgMatches<'a>) -> Vec<&'a str> {
    matches.values_of("env").map_or(Vec::new(), |v| v.collect())
}

fn get_sets<'a>(matches: &'a ArgMatches) -> Vec<Option<(String, String)>> {
    matches.values_of("set").map_or(Vec::new(), |v| {
        v.map(|v| split_line(v.to_string())).collect()
    })
}

fn get_dir<'a>(matches: &'a ArgMatches) -> Dir {
    if matches.is_present("merge_left") {
        Dir::L
    } else {
        Dir::R
    }
}
