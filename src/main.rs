use std::env;
use serde_json::json;
use serde_json::Value;
use rocket::fs::relative;
use tokio::runtime::Runtime;

use pankosmia_web;

fn do_rocket(conf: Value) {
    let rt = Runtime::new().unwrap();
    let builder = pankosmia_web::rocket(conf);
    rt.block_on(
        async move {
            let _ = builder.launch().await;
        }
    );
}

pub fn main() {
    let args: Vec<String> = env::args().collect();
    let mut working_dir = "".to_string();
    if args.len() == 2 {
        working_dir = args[1].clone();
    };
    let webfont_path = relative!("./webfonts");
    let app_setup_path = relative!("./setup/app_setup.json");
    let local_setup_path = relative!("./setup/local_setup.json");
    let app_resources_path = relative!("");
    let conf = json!({
        "working_dir": working_dir,
        "webfont_path": webfont_path,
        "app_setup_path": app_setup_path,
        "local_setup_path": local_setup_path,
        "app_resources_path": app_resources_path,
    });
    let _ = do_rocket(conf);
}