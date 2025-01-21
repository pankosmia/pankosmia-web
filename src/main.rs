use std::env;
use serde_json::json;
use rocket::fs::relative;
mod lib;
pub fn main() -> () {
    let args: Vec<String> = env::args().collect();
    let mut working_dir = "".to_string();
    if args.len() == 2 {
        working_dir = args[1].clone();
    };
    let webfont_path = relative!("../webfonts");
    let conf = json!({
        "working_dir": working_dir,
        "webfont_path": webfont_path
    });
    lib::rocket(conf).launch();
}