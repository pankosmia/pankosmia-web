use copy_dir::copy_dir;
use std::fs;
use std::io::Write;
use std::path::Path;
use serde_json::{json, Map, Value};
use crate::utils::client::Clients;
use crate::utils::files::{copy_and_customize_webfont_css, customize_and_copy_template_file};
use crate::utils::paths::{
    user_settings_path,
    app_state_path,
    app_setup_path,
    maybe_os_quoted_path_str,
    os_slash_str,
};
use crate::utils::files::{
    load_json,
    load_and_substitute_json,
};
use crate::utils::json::get_string_value_by_key;
pub(crate) fn initialize_working_dir(app_resources_dir_path: &String, working_dir_path: &String) -> () {
    // Make working dir
    match fs::create_dir_all(working_dir_path) {
        Ok(_) => {}
        Err(e) => {
            panic!("Could not create working dir '{}': {}", working_dir_path, e);
        }
    };
    // Copy user_settings file to working dir
    let user_settings_template_path = format!("{}/templates/user_settings.json", &app_resources_dir_path);
    let user_settings = user_settings_path(working_dir_path);
    match customize_and_copy_template_file(&user_settings_template_path, &user_settings, &working_dir_path, &app_resources_dir_path) {
        Ok(_) => {}
        Err(e) => {
            panic!("Error while copying user settings template file {} to {}: {}", user_settings_template_path, user_settings, e);
        }
    }
    // Copy app_state file to working dir
    let app_state_template_path = format!("{}templates/app_state.json", &app_resources_dir_path);
    let app_state = app_state_path(working_dir_path);
    match customize_and_copy_template_file(&app_state_template_path, &app_state, &working_dir_path, &app_resources_dir_path) {
        Ok(_) => {}
        Err(e) => {
            panic!("Error while copying app state template file {} to {}: {}", app_state_template_path, app_state, e);
        }
    }
}

pub(crate) fn load_configs(working_dir_path: &String, launch_config: &Value) -> (Value, Value, Value) {
    // Load local setup JSON
    let local_setup_path = get_string_value_by_key(&launch_config, "local_setup_path");
    let local_setup_json = match load_json(local_setup_path.as_str()) {
        Ok(json) => json,
        Err(e) => {
            panic!(
                "Could not read and parse local_json JSON file '{}': {}",
                local_setup_path,
                e
            );
        }
    };
    let local_pankosmia_path = get_string_value_by_key(&local_setup_json, "local_pankosmia_path");
    // Load app_setup JSON, substituting pankosmia path
    let app_setup_path = get_string_value_by_key(&launch_config, "app_setup_path");
    let pankosmia_dir = maybe_os_quoted_path_str(local_pankosmia_path.to_string());
    let app_setup_json = match load_and_substitute_json(app_setup_path, "%%PANKOSMIADIR%%", pankosmia_dir.as_str()) {
        Ok(json) => json,
        Err(e) => {
            panic!(
                "Could not read and parse substituted app setup JSON file '{}': {}",
                app_setup_path,
                e
            );
        }
    };
    // Load app state JSON
    let app_state = app_state_path(working_dir_path);
    let app_state_json = match load_json(&app_state) {
        Ok(json) => json,
        Err(e) => {
            panic!(
                "Could not read and parse app state JSON file '{}': {}",
                app_setup_path,
                e
            );
        }
    };
    // Load user settings JSON
    let user_settings = user_settings_path(working_dir_path);
    let user_settings_json = match load_json(&user_settings) {
        Ok(json) => json,
        Err(e) => {
            panic!(
                "Could not read and parse user settings JSON file '{}': {}",
                user_settings,
                e
            );
        }
    };
    (app_setup_json, user_settings_json, app_state_json)
}

pub(crate) fn maybe_make_repo_dir(repo_dir_path: &String) -> () {
    let repo_dir_path_exists = Path::new(&repo_dir_path).is_dir();
    if !repo_dir_path_exists {
        match fs::create_dir_all(&repo_dir_path) {
            Ok(_) => {}
            Err(e) => {
                panic!(
                    "Repo dir '{}' does not exist and could not be created: {}",
                    repo_dir_path, e
                );
            }
        };
    }
}

pub(crate) fn copy_and_customize_webfonts(template_path: &String, target_path: &String, user_settings: &Value) -> () {
    if !Path::new(&target_path).is_dir() {
        match copy_dir(template_path, target_path.clone()) {
            Ok(_) => {}
            Err(e) => {
                panic!(
                    "Could not copy web fonts to working directory from {}: {}",
                    template_path, e
                );
            }
        }
        let typography_json = match user_settings["typography"].as_object() {
            Some(json) => json,
            None => { panic!("Could not read typography from user_settings as object") }
        };
        let features_json = match typography_json["features"].as_object() {
            Some(json) => json,
            None => { panic!("Could not read features from user_settings as object") }
        };
        for font_name in features_json.keys() {
            let font_filename = format!("{}{}pankosmia-{}.css", &target_path, os_slash_str(), font_name);
            if Path::new(&font_filename).is_file() {
                match copy_and_customize_webfont_css(template_path, target_path, user_settings, font_name) {
                    Ok(_) => {},
                    Err(e) => { panic!("Could not customize webfont {}: {}", font_name, e) }
                }
            }
        }
    };
}

pub(crate) fn merged_clients(app_setup_json: &Value, user_settings_json: &Value) -> Vec<Value> {
    let mut client_records_merged_array: Vec<Value> = Vec::new();
    let app_client_records = app_setup_json["clients"].as_array().unwrap();
    for app_client_record in app_client_records.iter() {
        let mut record2 = app_client_record.clone();
        let mut_record = record2.as_object_mut().unwrap();
        let src_key = "src".to_string();
        let src_value = Value::from("App");
        mut_record.insert(src_key, src_value);
        client_records_merged_array.push(Value::Object(mut_record.clone()));
    }
    let my_client_records = user_settings_json["my_clients"].as_array().unwrap();
    for my_client_record in my_client_records.iter() {
        let mut record2 = my_client_record.clone();
        let mut_record = record2.as_object_mut().unwrap();
        let src_key = "src".to_string();
        let src_value = Value::from("User");
        mut_record.insert(src_key, src_value);
        client_records_merged_array.push(Value::Object(mut_record.clone()));
    }
    client_records_merged_array
}

pub(crate) fn build_client_record(client_record: &Value) -> Value {
    // Get requires from metadata
    let client_path = get_string_value_by_key(&client_record, "path");
    let client_metadata_path = format!("{}{}pankosmia_metadata.json", client_path, os_slash_str());
    let metadata_json = match load_json(client_metadata_path.as_str()) {
        Ok(json) => json,
        Err(e) => {
            panic!(
                "Could not read and parse metadata JSON file for '{}': {}",
                client_record,
                e
            );
        }
    };
    let mut debug_flag = false;
    let md_require = metadata_json["require"].as_object().unwrap();
    if md_require.contains_key("debug") {
        debug_flag = md_require.clone()["debug"].as_bool().unwrap();
    }
    let requires = json!({
            "net": md_require.clone()["net"].as_bool().unwrap(),
            "debug": debug_flag
        });
    // Get url from package.json
    let package_json_path = format!("{}{}package.json", get_string_value_by_key(&client_record, "path"), os_slash_str());
    let package_json = match load_json(package_json_path.as_str()) {
        Ok(json) => json,
        Err(e) => {
            panic!(
                "Could not read and parse package.json file for '{}': {}",
                client_record,
                e
            );
        }
    };
    // Build client record
    json!({
            "id": metadata_json["id"].as_str().unwrap(),
            "path": client_record["path"].as_str().unwrap(),
            "url": package_json["homepage"].as_str().unwrap(),
            "requires": requires,
            "exclude_from_menu": metadata_json["exclude_from_menu"].as_bool().unwrap_or_else(|| false),
            "exclude_from_dashboard": metadata_json["exclude_from_dashboard"].as_bool().unwrap_or_else(|| false),
            "src": client_record["src"].as_str().unwrap(),
        })
}

pub(crate) fn build_clients_and_i18n(clients_merged_array: Vec<Value>, app_resources_path: &String, working_dir_path: &String) -> Clients {
    let clients_value = serde_json::to_value(clients_merged_array).unwrap();
    let clients: Clients = match serde_json::from_value(clients_value) {
        Ok(v) => v,
        Err(e) => {
            panic!(
                "Could not parse clients array in settings file '{}' as client records: {}",
                app_setup_path(&working_dir_path), e
            );
        }
    };
    let i18n_template_path = format!("{}templates/i18n.json", app_resources_path);
    let mut i18n_json_map: Map<String, Value> = match fs::read_to_string(&i18n_template_path) {
        Ok(it) => match serde_json::from_str(&it) {
            Ok(i) => i,
            Err(e) => {
                panic!(
                    "Could not parse i18n template {} JSON as map: {}\n{}",
                    &i18n_template_path, e, it
                );
            }
        },
        Err(e) => {
            panic!("Could not read i18n template {}: {}", i18n_template_path, e);
        }
    };
    let mut i18n_pages_map = Map::new();
    let mut found_main = false;
    let mut locked_clients = clients.lock().unwrap().clone();
    let inner_clients = &mut *locked_clients;
    // Iterate over clients to build i18n
    for client_record in inner_clients {
        if !Path::new(&client_record.path.clone()).is_dir() {
            panic!(
                "Client path {} from app_setup file {} is not a directory",
                client_record.path, app_setup_path(&working_dir_path)
            );
        }
        let build_path = format!("{}/build", client_record.path.clone());
        if !Path::new(&build_path.clone()).is_dir() {
            panic!(
                "Client build path within {} from app_setup file {} does not exist or is not a directory. Do you need to build the client {}?",
                client_record.path.clone(),
                app_setup_path(&working_dir_path),
                client_record.id
            );
        }
        let client_metadata_path = format!("{}{}pankosmia_metadata.json", &client_record.path, os_slash_str());
        let metadata_json = match load_json(client_metadata_path.as_str()) {
            Ok(json) => json,
            Err(e) => {
                panic!(
                    "Could not read and parse pankosmia metadata file for '{}': {}",
                    client_record.id,
                    e
                );
            }
        };
        let metadata_id = get_string_value_by_key(&metadata_json, "id");
        let metadata_i18n = metadata_json["i18n"].clone();
        i18n_pages_map.insert(metadata_id.clone(), metadata_i18n);
        if client_record.url.clone() == "/clients/main".to_string() {
            found_main = true;
        }
    }
    i18n_json_map.insert(
        "pages".to_string(),
        Value::Object(i18n_pages_map),
    );
    let i18n_target_path = format!("{}{}i18n.json", &working_dir_path, os_slash_str());
    let i18n_file_exists = Path::new(&i18n_target_path).is_file();
    if !i18n_file_exists { // Do not overwrite for now
        let mut i18n_file_handle = match fs::File::create(&i18n_target_path) {
            Ok(h) => h,
            Err(e) => {
                panic!(
                    "Could not open target i18n file '{}': {}",
                    i18n_target_path, e
                );
            }
        };
        match i18n_file_handle.write_all(
            Value::Object(i18n_json_map)
                .to_string()
                .as_bytes(),
        ) {
            Ok(_) => {}
            Err(e) => {
                panic!(
                    "Could not write target i18n file to '{}': {}",
                    i18n_target_path, e
                );
            }
        }
    }
    // Throw if no main found
    if !found_main {
        panic!("Could not find a client registered at /main among clients in settings file");
    };
    clients
}