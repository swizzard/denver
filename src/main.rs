extern crate clap;

use denver::cmd::{denver_arg, get_args};
use denver::run_with_env;

fn main() {
    let matches = denver_arg().get_matches();
    let (cmd, e) = get_args(&matches);
    run_with_env(cmd, e);
}
