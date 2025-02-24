#[macro_use]
#[cfg(test)]
mod tests;

use copy_dir::copy_dir;
use rocket::fs::{relative, FileServer};
use rocket::{catchers, routes, Build, Rocket};
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, VecDeque};
use std::io::Write;
use std::path::{Path};
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::{env, fs};
mod structs;
use crate::structs::AppSettings;
mod utils;
use crate::utils::paths::{
    os_slash_str,
    maybe_os_quoted_path_str,
    home_dir_string
};
use crate::utils::client::Clients;
mod static_vars;
use crate::static_vars::{DEBUG_IS_ENABLED, NET_IS_ENABLED};
mod endpoints;


type MsgQueue = Arc<Mutex<VecDeque<String>>>;

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
                println!("Could not create working dir '{}': {}", working_dir_path, e);
                exit(1);
            }
        };
        // Copy user_settings file to working dir
        let user_settings_template_path = relative!("./templates/user_settings.json");
        let user_settings_json_string = match fs::read_to_string(&user_settings_template_path) {
            Ok(s) => maybe_os_quoted_path_str(s.replace("%%WORKINGDIR%%", &working_dir_path)),
            Err(e) => {
                println!(
                    "Could not read user settings template file '{}': {}",
                    user_settings_template_path, e
                );
                exit(1);
            }
        };
        let mut file_handle = match fs::File::create(&user_settings_path) {
            Ok(h) => h,
            Err(e) => {
                println!(
                    "Could not open user_settings file '{}' to write default: {}",
                    user_settings_path, e
                );
                exit(1);
            }
        };
        match file_handle.write_all(&user_settings_json_string.as_bytes()) {
            Ok(_) => {}
            Err(e) => {
                println!(
                    "Could not write default user_settings file to '{}: {}'",
                    user_settings_path, e
                );
                exit(1);
            }
        }
        // Copy app_state file to working dir
        let app_state_template_path = relative!("./templates/app_state.json");
        let app_state_json_string = match fs::read_to_string(&app_state_template_path) {
            Ok(s) => maybe_os_quoted_path_str(s.replace("%%WORKINGDIR%%", &working_dir_path)),
            Err(e) => {
                println!(
                    "Could not read app state template file '{}': {}",
                    user_settings_template_path, e
                );
                exit(1);
            }
        };
        let mut file_handle = match fs::File::create(&app_state_path) {
            Ok(h) => h,
            Err(e) => {
                println!(
                    "Could not open app_state file '{}' to write default: {}",
                    app_state_path, e
                );
                exit(1);
            }
        };
        match file_handle.write_all(&app_state_json_string.as_bytes()) {
            Ok(_) => {}
            Err(e) => {
                println!(
                    "Could not write default app_state file to '{}: {}'",
                    app_state_path, e
                );
                exit(1);
            }
        }
    }
    // Try to load local setup JSON
    let local_setup_path = launch_config["local_setup_path"].as_str().unwrap();
    let local_setup_json_string = match fs::read_to_string(&local_setup_path) {
        Ok(s) => s,
        Err(e) => {
            println!(
                "Could not read local setup file '{}': {}",
                local_setup_path, e
            );
            exit(1);
        }
    };
    let local_setup_json: Value = match serde_json::from_str(local_setup_json_string.as_str()) {
        Ok(j) => j,
        Err(e) => {
            println!(
                "Could not parse local setup file '{}': {}",
                local_setup_path, e
            );
            exit(1);
        }
    };
    let local_pankosmia_path = local_setup_json["local_pankosmia_path"].as_str().unwrap();
    // Try to load app_setup JSON, substituting pankosmia path
    let app_setup_path = launch_config["app_setup_path"].as_str().unwrap();
    let mut app_setup_json_string = match fs::read_to_string(&app_setup_path) {
        Ok(s) => s,
        Err(e) => {
            println!("Could not read app_setup file '{}': {}", app_setup_path, e);
            exit(1);
        }
    };
    app_setup_json_string = app_setup_json_string.replace(
        "%%PANKOSMIADIR%%",
        maybe_os_quoted_path_str(local_pankosmia_path.to_string()).as_str(),
    );
    let app_setup_json: Value = match serde_json::from_str(app_setup_json_string.as_str()) {
        Ok(j) => j,
        Err(e) => {
            println!("Could not parse app_setup file '{}': {}", app_setup_path, e);
            exit(1);
        }
    };
    // Try to load app state JSON
    let app_state_json_string = match fs::read_to_string(&app_state_path) {
        Ok(s) => s,
        Err(e) => {
            println!("Could not read app_state file '{}': {}", app_state_path, e);
            exit(1);
        }
    };
    let app_state_json: Value = match serde_json::from_str(app_state_json_string.as_str()) {
        Ok(j) => j,
        Err(e) => {
            println!("Could not parse app_state file '{}': {}", app_state_path, e);
            exit(1);
        }
    };
    // Try to load user settings JSON
    let user_settings_json_string = match fs::read_to_string(&user_settings_path) {
        Ok(s) => s,
        Err(e) => {
            println!(
                "Could not read user_settings file '{}': {}",
                user_settings_path, e
            );
            exit(1);
        }
    };
    let user_settings_json: Value = match serde_json::from_str(user_settings_json_string.as_str()) {
        Ok(j) => j,
        Err(e) => {
            println!(
                "Could not parse user_settings file '{}': {}",
                user_settings_path, e
            );
            exit(1);
        }
    };
    // Find or make repo_dir
    let repo_dir_path = match user_settings_json["repo_dir"].as_str() {
        Some(v) => v.to_string(),
        None => {
            println!(
                "Could not parse repo_dir in user_settings file '{}' as a string",
                user_settings_path
            );
            exit(1);
        }
    };
    let repo_dir_path_exists = Path::new(&repo_dir_path).is_dir();
    if !repo_dir_path_exists {
        match fs::create_dir_all(&repo_dir_path) {
            Ok(_) => {}
            Err(e) => {
                println!(
                    "Repo dir '{}' doe not exist and could not be created: {}",
                    repo_dir_path, e
                );
                exit(1);
            }
        };
    }
    // Copy web fonts from path in local config
    let template_webfonts_dir_path = launch_config["webfont_path"].as_str().unwrap();
    let webfonts_dir_path = working_dir_path.clone() + os_slash_str() + "webfonts";
    if !Path::new(&webfonts_dir_path).is_dir() {
        match copy_dir(template_webfonts_dir_path, webfonts_dir_path.clone()) {
            Ok(_) => {}
            Err(e) => {
                println!(
                    "Could not copy web fonts to working directory from {}: {}",
                    template_webfonts_dir_path, e
                );
                exit(1);
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
        let client_metadata_path = client_record["path"].as_str().unwrap().to_string()
            + os_slash_str()
            + "pankosmia_metadata.json";
        let metadata_json: Value = match fs::read_to_string(&client_metadata_path) {
            Ok(mt) => match serde_json::from_str(&mt) {
                Ok(m) => m,
                Err(e) => {
                    println!(
                        "Could not parse metadata file {} as JSON: {}\n{}",
                        &client_metadata_path, e, mt
                    );
                    exit(1);
                }
            },
            Err(e) => {
                println!(
                    "Could not read metadata file {}: {}",
                    client_metadata_path, e
                );
                exit(1);
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
        let package_json_path =
            client_record["path"].as_str().unwrap().to_string() + os_slash_str() + "package.json";
        let package_json: Value = match fs::read_to_string(&package_json_path) {
            Ok(pj) => match serde_json::from_str(&pj) {
                Ok(p) => p,
                Err(e) => {
                    println!(
                        "Could not parse package.json file {} as JSON: {}\n{}",
                        &package_json_path, e, pj
                    );
                    exit(1);
                }
            },
            Err(e) => {
                println!(
                    "Could not read package.json file {}: {}",
                    package_json_path, e
                );
                exit(1);
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
    let i18n_template_path = relative!("./templates").to_string() + os_slash_str() + "i18n.json";
    let mut i18n_json_map: Map<String, Value> = match fs::read_to_string(&i18n_template_path) {
        Ok(it) => match serde_json::from_str(&it) {
            Ok(i) => i,
            Err(e) => {
                println!(
                    "Could not parse i18n template {} as JSON: {}\n{}",
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
            println!(
                "Client path {} from app_setup file {} is not a directory",
                client_record.path, app_setup_path
            );
            exit(1);
        }
        let build_path = format!("{}/build", client_record.path.clone());
        if !Path::new(&build_path.clone()).is_dir() {
            println!("Client build path within {} from app_setup file {} does not exist or is not a directory", client_record.path.clone(), app_setup_path);
            exit(1);
        }
        let client_metadata_path =
            client_record.path.clone() + os_slash_str() + "pankosmia_metadata.json";
        let metadata_json: Value = match fs::read_to_string(&client_metadata_path) {
            Ok(mt) => match serde_json::from_str(&mt) {
                Ok(m) => m,
                Err(e) => {
                    println!(
                        "Could not parse metadata file {} as JSON: {}\n{}",
                        &client_metadata_path, e, mt
                    );
                    exit(1);
                }
            },
            Err(e) => {
                println!(
                    "Could not read metadata file {}: {}",
                    client_metadata_path, e
                );
                exit(1);
            }
        };
        let metadata_id = metadata_json["id"].as_str().unwrap().to_string();
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
    let i18n_target_path = working_dir_path.clone() + os_slash_str() + "i18n.json";
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
        .mount("/notifications", routes![endpoints::sse::notifications_stream,])
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
                endpoints::i18n::raw_i18n,
                endpoints::i18n::negotiated_i18n,
                endpoints::i18n::flat_i18n,
                endpoints::i18n::untranslated_i18n
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
