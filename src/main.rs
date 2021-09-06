extern crate clap;

use denver::cmd::{denver_arg, get_args};
use denver::run_with_env;

fn main() -> Result<(), String> {
    let matches = denver_arg().get_matches();
    let (cmd, e) = get_args(&matches).map_err(|e| {
        let s = e.to_string();
        s
    })?;
    run_with_env(cmd, e).map_err(|e| {
        let s = e.to_string();
        s
    })
}
