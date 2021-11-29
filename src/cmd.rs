use crate::{get_from, merge_envs, set, split_line, Dir};
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
                .help("Set one-time vars (clobbers -f)")
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
            Arg::with_name("from")
                .short("f")
                .long("from")
                .value_name("KEY=ENV")
                .help("Set a variable from a specific environment")
                .takes_value(true)
                .multiple(true),
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
    let froms = get_froms(matches)?;
    let sets = get_sets(matches);
    let dir = get_dir(matches);
    let mut e = merge_envs(ens, dir)?;
    set_froms(froms, &mut e)?;
    set_sets(sets, &mut e);
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

fn get_froms<'a>(matches: &'a ArgMatches<'a>) -> io::Result<Vec<Option<(String, String)>>> {
    let fs = matches.values_of("from").map_or(Vec::new(), |v| {
        v.map(|v| split_line(v.to_string())).collect()
    });
    fs.into_iter()
        .filter(|v| v.is_some())
        .map(|v| v.unwrap())
        .map(|(k, v)| get_from(v, k))
        .collect()
}

fn set_froms(
    froms: Vec<Option<(String, String)>>,
    mut e: &mut HashMap<String, String>,
) -> io::Result<()> {
    for (k, v) in froms.into_iter().flatten() {
        set(&mut e, k, v)
    }
    Ok(())
}

fn get_sets(matches: &ArgMatches) -> Vec<Option<(String, String)>> {
    matches
        .values_of("set")
        .map_or(Vec::new(), |v| {
            v.map(|v| split_line(v.to_string())).collect()
        })
        .into_iter()
        .filter(|v| v.is_some())
        .collect()
}

fn set_sets(sets: Vec<Option<(String, String)>>, mut e: &mut HashMap<String, String>) {
    for (k, v) in sets.into_iter().flatten() {
        set(&mut e, k, v);
    }
}

fn get_dir(matches: &ArgMatches) -> Dir {
    if matches.is_present("merge_left") {
        Dir::L
    } else {
        Dir::R
    }
}
