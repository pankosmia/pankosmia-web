#[macro_use]
#[cfg(test)]
mod tests;

#[doc(hidden)]
use rocket::{Build, Rocket};
use serde_json::{Value};
use std::env;
use std::collections::{VecDeque};
use std::path::Path;
use std::sync::{Arc, Mutex};

mod structs;
mod utils;
use crate::utils::paths::{os_slash_str, home_dir_string, webfonts_path};
use crate::utils::bootstrap::{copy_and_customize_webfonts, initialize_working_dir, load_configs, maybe_make_repo_dir, merged_clients, build_client_record, build_clients_and_i18n};
use crate::utils::json::get_string_value_by_key;
use crate::utils::launch::{add_catchers, add_routes, add_app_settings, add_static_routes};
mod static_vars;
use crate::static_vars::{DEBUG_IS_ENABLED, NET_IS_ENABLED};
pub mod endpoints;

type MsgQueue = Arc<Mutex<VecDeque<String>>>;

pub fn rocket(launch_config: Value) -> Rocket<Build> {
    println!("OS = '{}'", env::consts::OS);

    // Default workspace path
    let root_path = home_dir_string() + os_slash_str();
    let mut working_dir_path = format!("{}pankosmia_working", root_path.clone());

    // Override default if another value is supplied
    let launch_working_dir = get_string_value_by_key(&launch_config, "working_dir");
    if launch_working_dir.len() > 3 { // Try not to mangle entire FS with empty path strings
        working_dir_path = launch_working_dir.clone();
    };

    // Make new working dir if necessary
    if !Path::new(&working_dir_path).is_dir() {
        initialize_working_dir(&working_dir_path);
    }

    // Load the config JSONs
    let (app_setup_json, user_settings_json, app_state_json) = load_configs(&working_dir_path, &launch_config);

    // Find or make repo_dir
    let repo_dir_path = get_string_value_by_key(&user_settings_json, "repo_dir");
    maybe_make_repo_dir(&repo_dir_path);

    // Copy web fonts from path in local config
    let template_webfonts_dir_path = get_string_value_by_key(&launch_config, "webfont_path");
    let webfonts_dir_path = webfonts_path(&working_dir_path);
    copy_and_customize_webfonts(template_webfonts_dir_path, &webfonts_dir_path, &user_settings_json);

    // Merge client config (from app setup and user settings) into settings JSON
    let client_records_merged_array = merged_clients(&app_setup_json, &user_settings_json);

    // Construct clients as Values
    let mut clients_merged_array: Vec<Value> = Vec::new();
    for client_record in client_records_merged_array.iter() {
        clients_merged_array.push(build_client_record(&client_record));
    }
    // Build complete clients with i18n
    let clients = build_clients_and_i18n(clients_merged_array, &working_dir_path);

    // *** LAUNCH ROCKET ***
    let mut my_rocket = rocket::build();

    // Error handlers
    my_rocket = add_catchers(my_rocket);

    // Routes
    my_rocket = add_routes(my_rocket);
    let client_vec = clients.lock().unwrap().clone();
    my_rocket = add_static_routes(my_rocket, client_vec, &webfonts_dir_path);

    // State
    my_rocket = add_app_settings(my_rocket, &repo_dir_path, &working_dir_path, &user_settings_json, &app_state_json);
    let msg_queue = MsgQueue::new(Mutex::new(VecDeque::new()));
    my_rocket = my_rocket.manage(msg_queue).manage(clients);

    my_rocket
}
