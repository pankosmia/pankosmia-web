use std::collections::BTreeMap;
use std::sync::Mutex;
use rocket::{catchers, routes, Build, Rocket};
use rocket::fs::FileServer;
use serde_json::{json, Value};
use crate::endpoints;
use crate::structs::{AppSettings, Client};
use crate::utils::paths::os_slash_str;

pub(crate) fn add_routes(rocket_instance: Rocket<Build>) -> Rocket<Build> {
    rocket_instance
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
}

pub(crate) fn add_catchers(rocket_instance: Rocket<Build>) -> Rocket<Build> {
    rocket_instance
        .register("/", catchers![
            endpoints::error::not_found_catcher,
            endpoints::error::default_catcher
        ])
}

pub(crate) fn add_app_settings(rocket_instance: Rocket<Build>, repo_dir_path: &String, working_dir_path: &String, user_settings_json: &Value, app_state_json: &Value) -> Rocket<Build> {
    rocket_instance.manage(AppSettings {
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
                panic!("Could not read typography from parsed user settings file");
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
}

pub(crate) fn add_static_routes(rocket_instance: Rocket<Build>, client_vec: Vec<Client>, webfonts_dir_path: &String) -> Rocket<Build> {
    let mut my_rocket = rocket_instance.mount("/webfonts", FileServer::from(webfonts_dir_path.clone()));
    for client_record in client_vec {
        my_rocket = my_rocket.mount(
            client_record.url.clone(),
            FileServer::from(client_record.path.clone() + os_slash_str() + "build"),
        );
    }
    my_rocket
}