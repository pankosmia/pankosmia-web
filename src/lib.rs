#[macro_use]
#[cfg(test)]
mod tests;

#[doc(hidden)]
use rocket::{Build, Rocket};
use serde_json::{json, Value};
use std::env;
use std::collections::{BTreeMap, VecDeque};
use std::path::Path;
use std::sync::{Arc, Mutex};

mod structs;
mod utils;
use crate::utils::paths::{os_slash_str, home_dir_string, webfonts_path, source_local_setup_path};
use crate::utils::bootstrap::{copy_and_customize_webfonts, initialize_working_dir, load_configs, maybe_make_repo_dir, merged_clients, build_client_record, build_clients_and_i18n};
use crate::utils::json::get_string_value_by_key;
use crate::utils::launch::{add_catchers, add_routes, add_app_settings, add_static_routes};
use crate::utils::files::load_json;
pub mod endpoints;
mod static_vars;
#[allow(unused_imports)]
use crate::static_vars::{DEBUG_IS_ENABLED, NET_IS_ENABLED};
use crate::structs::ClientConfigSection;

#[warn(unused_imports)]

type MsgQueue = Arc<Mutex<VecDeque<String>>>;

pub fn rocket(launch_config: Value) -> Rocket<Build> {
    println!("OS = '{}'", env::consts::OS);

    // Get product JSON
    let binary_path = env::current_exe().unwrap();
    let binary_parent_dir_path = binary_path.parent().unwrap().parent().unwrap().to_str().unwrap();
    let product_path = format!(
        "{}{}lib{}app_resources{}product{}product.json",
        binary_parent_dir_path,
        os_slash_str(),
        os_slash_str(),
        os_slash_str(),
        os_slash_str(),
    );
    let product_json = match load_json(product_path.as_str()) {
        Ok(j) => j,
        Err(e) => panic!("Could not read and parse product json as {}: {}", product_path, e)
    };
    let product_short_name = product_json["short_name"].as_str().unwrap().to_string();
    println!("Product = {}", &product_short_name);

    // Maybe get client_config JSON
    let client_config_path = format!(
        "{}{}lib{}app_resources{}product{}client_config.json",
        binary_parent_dir_path,
        os_slash_str(),
        os_slash_str(),
        os_slash_str(),
        os_slash_str(),
    );
    let client_config_json = match load_json(client_config_path.as_str()) {
        Ok(j) => j,
        Err(_e) => {
            println!("WARNING: No client config file found");
            json!({})
        }
    };
    let mut client_config = BTreeMap::new();
    let client_config_json_object = client_config_json.as_object().expect("client config as object");
    for (section_k, section_v) in client_config_json_object {
        let section: Vec<ClientConfigSection> = serde_json::from_value(section_v.clone()).expect("client config as struct");
        client_config.insert(section_k.to_string(), section);
    }

    // Default workspace path
    let root_path = home_dir_string() + os_slash_str();
    let mut working_dir_path = format!(
        "{}pankosmia{}{}",
        root_path.clone(),
        os_slash_str(),
        &product_short_name
    );

    // Override default if another value is supplied
    let launch_working_dir = get_string_value_by_key(&launch_config, "working_dir");
    if launch_working_dir.len() > 3 { // Try not to mangle entire FS with empty path strings
        working_dir_path = launch_working_dir.clone();
    };

    // Make new working dir if necessary
    if !Path::new(&working_dir_path).is_dir() {
        let app_resources_dir = get_string_value_by_key(&launch_config, "app_resources_path");
        let local_setup_json = load_json(source_local_setup_path(app_resources_dir).as_str()).unwrap();
        initialize_working_dir(
            &local_setup_json["local_pankosmia_path"].as_str().unwrap().to_string(),
            &app_resources_dir,
            &working_dir_path
        );
    }

    // Load the config JSONs
    let (app_setup_json, user_settings_json, app_state_json) = load_configs(&working_dir_path, &launch_config);

    // Find or make repo_dir
    let repo_dir_path = get_string_value_by_key(&user_settings_json, "repo_dir");
    maybe_make_repo_dir(&repo_dir_path);
    // Check for app_resources_dir
    let app_resources_dir_path = match &user_settings_json["app_resources_dir"] {
        Value::Null => panic!("app_resources_dir does not exist in user_settings.json"),
        Value::String(s) => s.to_string(),
        _ => panic!("app_resources_dir exists in user_settings.json but is not a string"),
    };
    if !Path::new(&app_resources_dir_path).is_dir() {
        panic!("app_resources_dir setting '{}' in user_settings.json is not a directory", app_resources_dir_path);
    }

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
    let clients = build_clients_and_i18n(clients_merged_array, &app_resources_dir_path, &working_dir_path);

    // *** LAUNCH ROCKET ***
    let mut my_rocket = rocket::build();

    // Error handlers
    my_rocket = add_catchers(my_rocket);

    // Routes
    my_rocket = add_routes(my_rocket);
    let client_vec = clients.lock().unwrap().clone();
    my_rocket = add_static_routes(my_rocket, client_vec, &app_resources_dir_path, &webfonts_dir_path);

    // State
    my_rocket = add_app_settings(
        my_rocket,
        &repo_dir_path,
        &app_resources_dir_path,
        &working_dir_path,
        &user_settings_json,
        &app_state_json,
        &product_json,
        client_config
    );
    let msg_queue = MsgQueue::new(Mutex::new(VecDeque::new()));
    my_rocket = my_rocket.manage(msg_queue).manage(clients);

    my_rocket
}
