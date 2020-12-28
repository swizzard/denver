extern crate subprocess;
use std::collections::HashMap;
use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

use subprocess::{Popen, PopenConfig};

type DenvM = HashMap<String, String>;

pub fn read_lines(pth: &dyn AsRef<Path>) -> io::Result<io::Lines<io::BufReader<File>>> {
    let f = File::open(pth)?;
    Ok(io::BufReader::new(f).lines())
}

pub fn split_line(s: String) -> (String, String) {
    let mut sp = s.split("=");
    let a = sp.next().unwrap();
    let b = sp.next().unwrap();
    (a.to_string(), b.to_string())
}

pub fn to_denvm(v: Vec<(String, String)>) -> DenvM {
    v.into_iter().collect()
}

pub fn mk_env(de: DenvM) -> Option<Vec<(OsString, OsString)>> {
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

pub fn merge_l(l: DenvM, r: DenvM) -> DenvM {
    merge(r, l)
}

pub fn merge_r(l: DenvM, r: DenvM) -> DenvM {
    merge(l, r)
}

pub fn mk_cfg(env: DenvM) -> PopenConfig {
    PopenConfig {
        env: mk_env(env),
        ..Default::default()
    }
}

pub fn run_with_env(s: Vec<impl AsRef<OsStr>>, env: DenvM) -> () {
    let conf = mk_cfg(env);
    Popen::create(&s, conf).unwrap();
}

pub fn get_vars() -> DenvM {
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
    let lines: Vec<(String, String)> = lines
        .map(|line| line.map(|l| split_line(l)))
        .collect::<io::Result<Vec<(String, String)>>>()?;
    Ok(to_denvm(lines))
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
    fn test_run_with_env() {
        let de: DenvM = vec![("A".to_string(), "b".to_string())]
            .into_iter()
            .collect();
        let argv = vec!["echo", "$A"];
        println!("{:?}", argv);
        run_with_env(argv, de);
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
