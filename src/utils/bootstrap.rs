use std::fs;
use std::path::Path;
use rocket::fs::relative;
use serde_json::Value;
use crate::utils::files::{
    customize_and_copy_template_file,
};
use crate::utils::paths::{
    user_settings_path,
    app_state_path,
    maybe_os_quoted_path_str,
};
use crate::utils::files::{
    load_json,
    load_and_substitute_json
};
pub(crate) fn initialize_working_dir(working_dir_path: &String) -> () {
    // Make working dir
    match fs::create_dir_all(working_dir_path) {
        Ok(_) => {}
        Err(e) => {
            panic!("Could not create working dir '{}': {}", working_dir_path, e);
        }
    };
    // Copy user_settings file to working dir
    let user_settings_template_path = relative!("./templates/user_settings.json");
    let user_settings = user_settings_path(working_dir_path);
    match customize_and_copy_template_file(&user_settings_template_path, &user_settings, working_dir_path) {
        Ok(_) => {}
        Err(e) => {
            panic!("Error while copying user settings template file {} to {}: {}", user_settings_template_path, user_settings, e);
        }
    }
    // Copy app_state file to working dir
    let app_state_template_path = relative!("./templates/app_state.json");
    let app_state = app_state_path(working_dir_path);
    match customize_and_copy_template_file(&app_state_template_path, &app_state, working_dir_path) {
        Ok(_) => {}
        Err(e) => {
            panic!("Error while copying app state template file {} to {}: {}", app_state_template_path, app_state, e);
        }
    }
}

fn get_string_value_by_key<'a>(value: &'a Value, key: &'a str) -> &'a String {
    match &value[key] {
        Value::String(v) => v,
        _ => {
            panic!(
                "Could not get string value for key  '{}'",
                key
            );
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

pub(crate) fn maybe_make_repo_dir (repo_dir_path: &String) -> () {
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