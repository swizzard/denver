extern crate nix;
extern crate subprocess;

use std::collections::HashMap;
use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

use ctrlc::set_handler;
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use subprocess::{Popen, PopenConfig, PopenError};

pub mod cmd;

type DenvM = HashMap<String, String>;

fn read_lines(pth: &dyn AsRef<Path>) -> io::Result<io::Lines<io::BufReader<File>>> {
    let f = File::open(pth)?;
    Ok(io::BufReader::new(f).lines())
}

fn is_valid_ident(s: &str) -> bool {
    if let Some(fst) = s.chars().next() {
        fst.is_alphabetic()
    } else {
        false
    }
}

pub fn split_line(s: String) -> Option<(String, String)> {
    if is_valid_ident(&s) {
        s.split_once("=").map(|(a, b)| {
            (
                a.to_string(),
                b.trim_matches(|c| c == '\'' || c == '"').to_string(),
            )
        })
    } else {
        None
    }
}

fn to_denvm(v: Vec<(String, String)>) -> DenvM {
    v.into_iter().collect()
}

fn mk_env(de: DenvM) -> Option<Vec<(OsString, OsString)>> {
    Some(
        de.iter()
            .map(|(k, v)| (OsString::from(k), OsString::from(v)))
            .collect(),
    )
}

fn merge(fst: DenvM, snd: DenvM) -> DenvM {
    let mut merged = HashMap::new();
    for (k, v) in fst.into_iter() {
        merged.insert(k, v);
    }
    for (k, v) in snd.into_iter() {
        merged.insert(k, v);
    }
    merged
}

fn merge_l(l: DenvM, r: DenvM) -> DenvM {
    merge(r, l)
}

fn merge_r(l: DenvM, r: DenvM) -> DenvM {
    merge(l, r)
}

fn mk_cfg(env: DenvM) -> PopenConfig {
    PopenConfig {
        env: mk_env(env),
        ..Default::default()
    }
}

fn handle_pe(err: subprocess::PopenError) -> io::Error {
    match err {
        PopenError::IoError(e) => io::Error::new(io::ErrorKind::Other, e.to_string()),
        PopenError::LogicError(e) => io::Error::new(io::ErrorKind::Other, e),
        _ => io::Error::new(io::ErrorKind::Other, "Unknown error"),
    }
}

pub fn run_with_env(s: Vec<impl AsRef<OsStr>>, env: DenvM) -> io::Result<()> {
    let conf = mk_cfg(env);
    let proc = Popen::create(&s, conf).map_err(handle_pe)?;
    let pid = proc
        .pid()
        .ok_or(io::Error::new(io::ErrorKind::Other, "Process error"))?;
    let pid = Pid::from_raw(pid as i32);
    set_handler(move || {
        kill(pid, Signal::SIGINT).unwrap();
    })
    .unwrap();
    Ok(())
}

fn get_vars() -> DenvM {
    to_denvm(env::vars().into_iter().collect())
}

fn get_envf_path(name: Option<String>) -> io::Result<PathBuf> {
    let cwd = env::current_dir()?;
    let fname = {
        if let Some(n) = name {
            let lowered = n.to_lowercase();
            let mut ef = String::from(".");
            ef.push_str(&lowered);
            ef.push_str(".env");
            ef
        } else {
            ".env".to_string()
        }
    };
    Ok(cwd.join(fname))
}

fn name_to_denvm(name: Option<String>) -> io::Result<DenvM> {
    let pth = get_envf_path(name)?;
    let lines = read_lines(&pth)?;
    lines
        .map(|line| line.map(|l| split_line(l)))
        .filter_map(|x| x.transpose())
        .collect()
}

pub enum Dir {
    L,
    R,
}

pub fn merge_envs(names: Vec<&str>, dir: Dir) -> io::Result<DenvM> {
    let e = get_vars();
    let base = name_to_denvm(None)?;
    let mut es = vec![base];
    let f = {
        match dir {
            Dir::L => merge_l,
            Dir::R => merge_r,
        }
    };
    for name in names {
        let env = name_to_denvm(Some(name.to_string()))?;
        es.push(env);
    }
    Ok(es.into_iter().fold(e, f))
}

pub fn get_from(env_name: String, var_name: String) -> io::Result<Option<(String, String)>> {
    let e = name_to_denvm(Some(env_name.to_string()))?;
    match e.get(&var_name) {
        Some(v) => Ok(Some((var_name, v.to_string()))),
        None => Ok(None),
    }
}

pub fn set(env: &mut DenvM, k: String, v: String) -> () {
    env.insert(k, v);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_to_devm() {
        let s = "a=b\nc=d";
        let it = s
            .lines()
            .map(|s| s.to_owned())
            .map(|s| split_line(s))
            .filter_map(|x| x)
            .collect();
        let m = to_denvm(it);
        assert_eq!(m.get("a"), Some(&"b".to_string()));
        assert_eq!(m.get("c"), Some(&"d".to_string()));
    }

    #[test]
    fn test_to_denvm_cmts() {
        let s = "a=b\n# comment\nc=d\nx=";
        let it = s
            .lines()
            .map(|s| s.to_owned())
            .map(|s| split_line(s))
            .filter_map(|x| x)
            .collect();
        let m = to_denvm(it);
        assert_eq!(m.get("a"), Some(&"b".to_string()));
        assert_eq!(m.get("c"), Some(&"d".to_string()));
    }

    #[test]
    fn test_mk_env() {
        let de: DenvM = vec![
            ("a".to_string(), "b".to_string()),
            ("c".to_string(), "d".to_string()),
        ]
        .into_iter()
        .collect();
        let expected: Vec<(OsString, OsString)> = vec![
            (
                OsString::from("a".to_string()),
                OsString::from("b".to_string()),
            ),
            (
                OsString::from("c".to_string()),
                OsString::from("d".to_string()),
            ),
        ];
        if let Some(mut e) = mk_env(de) {
            e.sort();
            assert_eq!(e, expected);
        } else {
            unreachable!()
        }
    }

    #[test]
    fn test_merge() {
        let fst = vec![
            ("a".to_string(), "a_fst".to_string()),
            ("b".to_string(), "b_fst".to_string()),
        ]
        .into_iter()
        .collect();
        let snd = vec![("a".to_string(), "a_snd".to_string())]
            .into_iter()
            .collect();
        let m = merge(fst, snd);
        assert_eq!(m.get("a"), Some(&"a_snd".to_string()));
        assert_eq!(m.get("b"), Some(&"b_fst".to_string()));
    }

    #[test]
    fn test_run_with_env() -> io::Result<()> {
        let de: DenvM = vec![("A".to_string(), "b".to_string())]
            .into_iter()
            .collect();
        let argv = vec!["echo", "$A"];
        println!("{:?}", argv);
        run_with_env(argv, de)?;
        Ok(())
    }

    #[test]
    fn test_get_enf_path_none() -> io::Result<()> {
        let p = get_envf_path(None)?;
        assert!(p.ends_with(".env"));
        Ok(())
    }

    #[test]
    fn test_get_envf_path_some() -> io::Result<()> {
        let p = get_envf_path(Some("dev".to_string()))?;
        assert!(p.ends_with(".dev.env"));
        let p = get_envf_path(Some("DEV".to_string()))?;
        assert!(p.ends_with(".dev.env"));
        Ok(())
    }
}
