#[macro_use]
#[cfg(test)]
mod tests;

#[doc(hidden)]
use copy_dir::copy_dir;
use rocket::fs::{relative, FileServer};
use rocket::{catchers, routes, Build, Rocket};
use serde_json::{json, Map, Value};
use std;
use std::{env, fs};
use std::collections::{BTreeMap, VecDeque};
use std::io::{Write};
use std::path::{Path};
use std::process::exit;
use std::sync::{Arc, Mutex};

mod structs;
use crate::structs::AppSettings;
mod utils;
use crate::utils::paths::{
    os_slash_str,
    maybe_os_quoted_path_str,
    home_dir_string,
};
use crate::utils::client::Clients;
mod static_vars;
use crate::static_vars::{DEBUG_IS_ENABLED, NET_IS_ENABLED};
pub mod endpoints;

type MsgQueue = Arc<Mutex<VecDeque<String>>>;

fn customize_and_copy_template_file(from_path: &str, to_path: &String, working_dir: &String) -> Result<(), std::io::Error> {
    let json_string = fs::read_to_string(&from_path)?;
    let quoted_json_string = maybe_os_quoted_path_str(json_string.replace("%%WORKINGDIR%%", &working_dir));
    let mut file_handle = fs::File::create(&to_path)?;
    file_handle.write_all(&quoted_json_string.as_bytes())?;
    Ok(())
}

fn load_json(from_path: &str) -> Result<Value, std::io::Error> {
    let json_string = fs::read_to_string(&from_path)?;
    Ok(serde_json::from_str(json_string.as_str())?)
}

fn load_and_substitute_json(from_path: &str, from_text: &str, to_text: &str) -> Result<Value, std::io::Error> {
    let mut json_string = fs::read_to_string(&from_path)?;
    json_string = json_string.replace(from_text, to_text);
    Ok(serde_json::from_str(json_string.as_str())?)
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

pub fn rocket(launch_config: Value) -> Rocket<Build> {
    println!("OS = '{}'", env::consts::OS);
    // Set up managed state;
    let msg_queue = MsgQueue::new(Mutex::new(VecDeque::new()));
    // Get settings path, default to well-known homedir location
    let root_path = home_dir_string() + os_slash_str();
    let mut working_dir_path = root_path.clone() + "pankosmia_working";
    if launch_config["working_dir"].as_str().unwrap().len() > 0 {
        working_dir_path = launch_config["working_dir"].as_str().unwrap().to_string();
    };
    let user_settings_path = format!("{}/user_settings.json", working_dir_path);
    let app_state_path = format!("{}/app_state.json", working_dir_path);
    let workspace_dir_exists = Path::new(&working_dir_path).is_dir();
    if !workspace_dir_exists {
        // Make working dir, argument is webfonts dir
        match fs::create_dir_all(&working_dir_path) {
            Ok(_) => {}
            Err(e) => {
                panic!("Could not create working dir '{}': {}", working_dir_path, e);
            }
        };
        // Copy user_settings file to working dir
        let user_settings_template_path = relative!("./templates/user_settings.json");
        match customize_and_copy_template_file(&user_settings_template_path, &user_settings_path, &working_dir_path) {
            Ok(_) => {},
            Err(e) => {
                panic!("Error while copying user settings template file {} to {}: {}", user_settings_template_path, user_settings_path, e);
            }
        }
        // Copy app_state file to working dir
        let app_state_template_path = relative!("./templates/app_state.json");
        match customize_and_copy_template_file(&app_state_template_path, &app_state_path, &working_dir_path) {
            Ok(_) => {},
            Err(e) => {
                panic!("Error while copying app state template file {} to {}: {}", app_state_template_path, app_state_path, e);
            }
        }
    }
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
    let app_state_json = match load_json(&app_state_path) {
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
    let user_settings_json = match load_json(&user_settings_path) {
        Ok(json) => json,
        Err(e) => {
            panic!(
                "Could not read and parse user settings JSON file '{}': {}",
                user_settings_path,
                e
            );
        }
    };
    // Find or make repo_dir
    let repo_dir_path = match user_settings_json["repo_dir"].as_str() {
        Some(v) => v.to_string(),
        None => {
            panic!(
                "Could not parse repo_dir in user_settings file '{}' as a string",
                user_settings_path
            );
        }
    };
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
    // Copy web fonts from path in local config
    let template_webfonts_dir_path = get_string_value_by_key(&launch_config, "webfont_path");
    let webfonts_dir_path = format!("{}{}webfonts", &working_dir_path, os_slash_str());
    if !Path::new(&webfonts_dir_path).is_dir() {
        match copy_dir(template_webfonts_dir_path, webfonts_dir_path.clone()) {
            Ok(_) => {}
            Err(e) => {
                panic!(
                    "Could not copy web fonts to working directory from {}: {}",
                    template_webfonts_dir_path, e
                );
            }
        }
    };
    // Merge client config into settings JSON
    let mut clients_merged_array: Vec<Value> = Vec::new();
    let mut client_records_merged_array: Vec<Value> = Vec::new();
    let app_client_records = app_setup_json["clients"].as_array().unwrap();
    for app_client_record in app_client_records.iter() {
        client_records_merged_array.push(app_client_record.clone());
    }
    let my_client_records = user_settings_json["my_clients"].as_array().unwrap();
    for my_client_record in my_client_records.iter() {
        client_records_merged_array.push(my_client_record.clone());
    }
    for client_record in client_records_merged_array.iter() {
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
        clients_merged_array.push(json!({
            "id": metadata_json["id"].as_str().unwrap(),
            "path": client_record["path"].as_str().unwrap(),
            "url": package_json["homepage"].as_str().unwrap(),
            "requires": requires,
            "exclude_from_menu": metadata_json["exclude_from_menu"].as_bool().unwrap_or_else(|| false),
            "exclude_from_dashboard": metadata_json["exclude_from_dashboard"].as_bool().unwrap_or_else(|| false)
        }));
    }
    let clients_value = serde_json::to_value(clients_merged_array).unwrap();
    // Process clients metadata to build clients and i18n
    let clients: Clients = match serde_json::from_value(clients_value) {
        Ok(v) => v,
        Err(e) => {
            println!(
                "Could not parse clients array in settings file '{}' as client records: {}",
                app_setup_path, e
            );
            exit(1);
        }
    };
    let i18n_template_path = format!("{}{}i18n.json", relative!("./templates"), os_slash_str());
    let mut i18n_json_map: Map<String, Value> = match fs::read_to_string(&i18n_template_path) {
        Ok(it) => match serde_json::from_str(&it) {
            Ok(i) => i,
            Err(e) => {
                println!(
                    "Could not parse i18n template {} JSON as map: {}\n{}",
                    &i18n_template_path, e, it
                );
                exit(1);
            }
        },
        Err(e) => {
            println!("Could not read i18n template {}: {}", i18n_template_path, e);
            exit(1);
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
                client_record.path, app_setup_path
            );
        }
        let build_path = format!("{}/build", client_record.path.clone());
        if !Path::new(&build_path.clone()).is_dir() {
            panic!(
                "Client build path within {} from app_setup file {} does not exist or is not a directory. Do you need to build the client {}?",
                client_record.path.clone(),
                app_setup_path,
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
    // Do not overwrite for now
    if !i18n_file_exists {
        let mut i18n_file_handle = match fs::File::create(&i18n_target_path) {
            Ok(h) => h,
            Err(e) => {
                println!(
                    "Could not open target i18n file '{}': {}",
                    i18n_target_path, e
                );
                exit(1);
            }
        };
        match i18n_file_handle.write_all(
            Value::Object(i18n_json_map)
                .to_string()
                .as_bytes(),
        ) {
            Ok(_) => {}
            Err(e) => {
                println!(
                    "Could not write target i18n file to '{}': {}",
                    i18n_target_path, e
                );
                exit(1);
            }
        }
    }
    // Throw if no main found
    if !found_main {
        println!("Could not find a client registered at /main among clients in settings file");
        exit(1);
    }

    let mut my_rocket = rocket::build()
        .register("/", catchers![
            endpoints::error::not_found_catcher,
            endpoints::error::default_catcher
        ])
        .manage(AppSettings {
            repo_dir: Mutex::new(repo_dir_path.clone()),
            working_dir: working_dir_path.clone(),
            languages: Mutex::new(
                user_settings_json["languages"]
                    .as_array()
                    .unwrap()
                    .into_iter()
                    .map(|i| {
                        i.as_str()
                            .expect("Non-string in user_settings language array")
                            .to_string()
                    })
                    .collect(),
            ),
            auth_tokens: match user_settings_json["auth_tokens"].clone() {
                Value::Object(v) => {
                    Mutex::new(serde_json::from_value(Value::Object(v)).unwrap())
                }
                _ => Mutex::new(BTreeMap::new()),
            },
            auth_requests: Mutex::new(BTreeMap::new()),
            gitea_endpoints: match user_settings_json["gitea_endpoints"].clone() {
                Value::Object(v) => {
                    serde_json::from_value(Value::Object(v)).unwrap()
                }
                _ => BTreeMap::new(),
            },
            typography: match user_settings_json["typography"].clone() {
                Value::Object(v) => {
                    serde_json::from_value(Value::Object(v)).unwrap()
                }
                _ => {
                    println!("Could not read typography from parsed user settings file");
                    exit(1)
                }
            },
            bcv: match app_state_json["bcv"].clone() {
                Value::Object(v) => {
                    serde_json::from_value(Value::Object(v)).unwrap()
                }
                _ => serde_json::from_value(json!({
                    "book_code": "TIT",
                    "chapter": 1,
                    "verse": 1
                }))
                    .unwrap(),
            },
        })
        .mount(
            "/",
            routes![
                endpoints::clients::redirect_root,
                endpoints::clients::serve_root_favicon,
                endpoints::clients::list_clients
            ],
        )
        .mount("/notifications", routes![
            endpoints::sse::notifications_stream
        ])
        .mount(
            "/settings",
            routes![
                endpoints::settings::get_languages,
                endpoints::settings::post_languages,
                endpoints::settings::get_auth_token,
                endpoints::settings::get_new_auth_token,
                endpoints::settings::get_typography,
                endpoints::settings::post_typography
            ],
        )
        .mount("/net", routes![
            endpoints::net::net_status,
            endpoints::net::net_enable,
            endpoints::net::net_disable
        ])
        .mount("/debug", routes![
            endpoints::debug::debug_status,
            endpoints::debug::debug_enable,
            endpoints::debug::debug_disable
        ])
        .mount(
            "/i18n",
            routes![
                endpoints::i18n::post_i18n,
                endpoints::i18n::raw_i18n,
                endpoints::i18n::negotiated_i18n,
                endpoints::i18n::flat_i18n,
                endpoints::i18n::untranslated_i18n,
                endpoints::i18n::used_languages
            ],
        )
        .mount("/navigation", routes![
            endpoints::navigation::get_bcv,
            endpoints::navigation::post_bcv
        ])
        .mount("/gitea", routes![
            endpoints::gitea::gitea_remote_repos,
            endpoints::gitea::get_gitea_endpoints,
            endpoints::gitea::gitea_proxy_login,
            endpoints::gitea::gitea_proxy_logout
        ])
        .mount(
            "/git",
            routes![
                endpoints::git::fetch_repo,
                endpoints::git::list_local_repos,
                endpoints::git::delete_repo,
                endpoints::git::add_and_commit,
                endpoints::git::git_status
            ],
        )
        .mount(
            "/burrito",
            routes![
                endpoints::burrito::raw_ingredient,
                endpoints::burrito::get_ingredient_prettified,
                endpoints::burrito::get_ingredient_as_usj,
                endpoints::burrito::post_ingredient_as_usj,
                endpoints::burrito::post_raw_ingredient,
                endpoints::burrito::raw_metadata,
                endpoints::burrito::summary_metadata
            ],
        )
        .mount("/webfonts", FileServer::from(webfonts_dir_path.clone()));
    let client_vec = clients.lock().unwrap().clone();
    for client_record in client_vec {
        my_rocket = my_rocket.mount(
            client_record.url.clone(),
            FileServer::from(client_record.path.clone() + os_slash_str() + "build"),
        );
    }
    my_rocket.manage(msg_queue).manage(clients)
}
