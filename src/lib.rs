#[macro_use]
#[cfg(test)]
mod tests;

#[doc(hidden)]
use rocket::fs::FileServer;
use rocket::{catchers, routes, Build, Rocket};
use serde_json::{json, Value};
use std;
use std::env;
use std::collections::{BTreeMap, VecDeque};
use std::path::Path;
use std::process::exit;
use std::sync::{Arc, Mutex};

mod structs;
use crate::structs::AppSettings;
mod utils;
use crate::utils::paths::{os_slash_str, home_dir_string, webfonts_path};
use crate::utils::bootstrap::{copy_webfonts, initialize_working_dir, load_configs, maybe_make_repo_dir, merged_clients, build_client_record, build_clients_and_i18n};
use crate::utils::json::get_string_value_by_key;
mod static_vars;
use crate::static_vars::{DEBUG_IS_ENABLED, NET_IS_ENABLED};
pub mod endpoints;

type MsgQueue = Arc<Mutex<VecDeque<String>>>;

pub fn rocket(launch_config: Value) -> Rocket<Build> {
    println!("OS = '{}'", env::consts::OS);
    // Set up managed state for message;
    let msg_queue = MsgQueue::new(Mutex::new(VecDeque::new()));

    // Default workspace path
    let root_path = home_dir_string() + os_slash_str();
    let mut working_dir_path = format!("{}pankosmia_working", root_path.clone());

    // Override default if another value is supplied
    let launch_working_dir = get_string_value_by_key(&launch_config, "working_dir");
    if launch_working_dir.len() > 3 { // Try not to mangle entire FS
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
    copy_webfonts(template_webfonts_dir_path, &webfonts_dir_path);

    // Merge client config into settings JSON
    let client_records_merged_array = merged_clients(&app_setup_json, &user_settings_json);

    // Construct clients as Values
    let mut clients_merged_array: Vec<Value> = Vec::new();
    for client_record in client_records_merged_array.iter() {
        clients_merged_array.push(build_client_record(&client_record));
    }
    // Build complete clients with i18n
    let clients = build_clients_and_i18n(clients_merged_array, &working_dir_path);

    // Launch Rocket
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
