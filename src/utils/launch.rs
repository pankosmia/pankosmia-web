use crate::endpoints;
use crate::structs::{AppSettings, Client, ProductSpec, ProjectIdentifier};
use crate::utils::paths::{os_slash_str, source_app_resources_path};
use rocket::fs::FileServer;
use rocket::{catchers, routes, Build, Rocket};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::sync::Mutex;

pub(crate) fn add_routes(rocket_instance: Rocket<Build>) -> Rocket<Build> {
    rocket_instance
        .mount(
            "/",
            routes![
                endpoints::clients::redirect_root,
                endpoints::clients::serve_root_favicon,
                endpoints::clients::list_clients,
                endpoints::clients::client_interfaces,
                endpoints::version::get_version,
            ],
        )
        .mount("/notifications", routes![
            endpoints::sse::notifications_stream
        ])
        .mount(
            "/settings",
            routes![
                endpoints::settings2::get_languages::get_languages,
                endpoints::settings2::post_languages::post_languages,
                endpoints::settings2::get_new_auth_token::get_new_auth_token,
                endpoints::settings2::get_typography::get_typography,
                endpoints::settings2::post_typography::post_typography,
                endpoints::settings2::post_typography_feature::post_typography_feature,
            ],
        )
        .mount("/net", routes![
            endpoints::atomics::net::net_status,
            endpoints::atomics::net::net_enable,
            endpoints::atomics::net::net_disable
        ])
        .mount("/debug", routes![
            endpoints::atomics::debug::debug_status,
            endpoints::atomics::debug::debug_enable,
            endpoints::atomics::debug::debug_disable
        ])
        .mount(
            "/i18n",
            routes![
                endpoints::i18n2::post_i18n::post_i18n,
                endpoints::i18n2::raw_i18n::raw_i18n,
                endpoints::i18n2::negotiated_i18n::negotiated_i18n,
                endpoints::i18n2::flat_i18n::flat_i18n,
                endpoints::i18n2::untranslated_i18n::untranslated_i18n,
                endpoints::i18n2::used_languages::used_languages
            ],
        )
        .mount("/navigation", routes![
            endpoints::navigation::get_bcv,
            endpoints::navigation::post_bcv
        ])
        .mount("/app-state", routes![
            endpoints::app_state::get_current_project,
            endpoints::app_state::post_current_project,
            endpoints::app_state::post_empty_current_project,
        ])
        .mount("/gitea", routes![
            endpoints::gitea2::gitea_remote_repos::gitea_remote_repos,
            endpoints::gitea2::get_gitea_endpoints::get_gitea_endpoints,
            endpoints::gitea2::gitea_proxy_login::gitea_proxy_login,
            endpoints::gitea2::gitea_proxy_logout::gitea_proxy_logout,
            endpoints::gitea2::get_my_collaborators::get_my_collaborators,
        ])
        .mount(
            "/git",
            routes![
                endpoints::git2::new_text_translation::new_text_translation_repo,
                endpoints::git2::new_bcv_resource::new_bcv_resource_repo,
                endpoints::git2::new_obs_resource::new_obs_resource_repo,
                endpoints::git2::clone_repo::clone_repo,
                endpoints::git2::list_local_repos::list_local_repos,
                endpoints::git2::delete_repo::delete_repo,
                endpoints::git2::add_and_commit::add_and_commit,
                endpoints::git2::status::git_status,
                endpoints::git2::new_scripture_book::new_scripture_book,
                endpoints::git2::copy_repo::copy_repo,
                endpoints::git2::remotes::list_remotes_for_repo,
                endpoints::git2::add_remote::add_remote_to_repo,
                endpoints::git2::delete_remote::delete_remote_from_repo,
                endpoints::git2::push::push_repo,
                endpoints::git2::log::log_repo,
                endpoints::git2::branches::list_branches_for_repo,
                endpoints::git2::set_branch::set_branch,
                endpoints::git2::create_and_set_branch::create_and_set_branch,
                endpoints::git2::pull_repo::pull_repo,
                endpoints::git2::new_bcv_resource_book::new_bcv_resource_book,
                endpoints::git2::new_tcore_resource::new_tcore_resource_repo
            ],
        )
        .mount(
            "/content-utils",
            routes![
                endpoints::content_utils2::list_content_templates::list_content_templates,
                endpoints::content_utils2::content_metadata_template::content_metadata_template,
                endpoints::content_utils2::list_content_template_filenames::list_content_template_filenames,
                endpoints::content_utils2::content_template::content_template,
                endpoints::content_utils2::list_versifications::list_versifications,
                endpoints::content_utils2::versification::versification
            ]
        )
        .mount(
            "/burrito",
            routes![
                endpoints::burrito2::raw_text_ingredient::raw_text_ingredient,
                endpoints::burrito2::raw_text_ingredients::raw_text_ingredients,
                endpoints::burrito2::raw_bytes_ingredient::raw_bytes_ingredient,
                endpoints::burrito2::get_ingredient_prettified::get_ingredient_prettified,
                endpoints::burrito2::post_raw_ingredient::post_raw_ingredient,
                endpoints::burrito2::post_bytes_ingredient::post_bytes_ingredient,
                endpoints::burrito2::post_delete_ingredient::post_delete_ingredient,
                endpoints::burrito2::post_delete_ingredients::post_delete_ingredients,
                endpoints::burrito2::post_copy_ingredient::copy_ingredient,
                endpoints::burrito2::post_revert_ingredient::post_revert_ingredient,
                endpoints::burrito2::raw_metadata::raw_metadata,
                endpoints::burrito2::summary_metadata::summary_metadata,
                endpoints::burrito2::summary_metadatas::summary_metadatas,
                endpoints::burrito2::get_repo_file_paths::get_repo_file_paths,
                endpoints::burrito2::audit::audit,
                endpoints::burrito2::post_remake_ingredients_metadata::remake_ingredients_metadata,
                endpoints::burrito2::post_zipped_ingredient::post_zipped_ingredient,
                endpoints::burrito2::get_zipped_ingredients::raw_zipped_ingredient
            ],
        )
    .mount(
        "/video",
        routes![
            endpoints::video::obs_para::obs_para_video,
            endpoints::video::obs_story::obs_story_video
        ]
    )
}

pub(crate) fn add_catchers(rocket_instance: Rocket<Build>) -> Rocket<Build> {
    rocket_instance.register(
        "/",
        catchers![
            endpoints::error::not_found_catcher,
            endpoints::error::default_catcher
        ],
    )
}

pub(crate) fn add_app_settings(
    rocket_instance: Rocket<Build>,
    repo_dir_path: &String,
    app_resources_dir_path: &String,
    working_dir_path: &String,
    user_settings_json: &Value,
    app_state_json: &Value,
    product_json: &Value,
) -> Rocket<Build> {
    rocket_instance.manage(AppSettings {
        repo_dir: Mutex::new(repo_dir_path.clone()),
        app_resources_dir: app_resources_dir_path.clone(),
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
            Value::Object(v) => Mutex::new(serde_json::from_value(Value::Object(v)).unwrap()),
            _ => Mutex::new(BTreeMap::new()),
        },
        auth_requests: Mutex::new(BTreeMap::new()),
        gitea_endpoints: match user_settings_json["gitea_endpoints"].clone() {
            Value::Object(v) => serde_json::from_value(Value::Object(v)).unwrap(),
            _ => BTreeMap::new(),
        },
        typography: match user_settings_json["typography"].clone() {
            Value::Object(v) => serde_json::from_value(Value::Object(v)).unwrap(),
            _ => {
                panic!("Could not read typography from parsed user settings file");
            }
        },
        bcv: match app_state_json["bcv"].clone() {
            Value::Object(v) => serde_json::from_value(Value::Object(v)).unwrap(),
            _ => serde_json::from_value(json!({
            "book_code": "TIT",
            "chapter": 1,
            "verse": 1
            }))
            .unwrap(),
        },
        current_project: match app_state_json["current_project"].clone() {
            Value::Object(p) => Mutex::new(Some(ProjectIdentifier {
                source: p["source"].as_str().unwrap().to_string(),
                organization: p["organization"].as_str().unwrap().to_string(),
                project: p["project"].as_str().unwrap().to_string(),
            })),
            _ => Mutex::new(None),
        },
        product: ProductSpec {
            name: product_json["name"].as_str().unwrap().to_string(),
            short_name: product_json["short_name"].as_str().unwrap().to_string(),
            version: product_json["version"].as_str().unwrap().to_string(),
            date_time: product_json["datetime"].as_str().unwrap().to_string(),
        },
    })
}

pub(crate) fn add_static_routes(
    rocket_instance: Rocket<Build>,
    client_vec: Vec<Client>,
    app_resources_path: &String,
    webfonts_dir_path: &String,
) -> Rocket<Build> {
    let mut my_rocket =
        rocket_instance.mount("/webfonts", FileServer::from(webfonts_dir_path.clone()));
    let app_resources_path = source_app_resources_path(&app_resources_path);
    my_rocket = my_rocket.mount("/app-resources", FileServer::from(app_resources_path));
    for client_record in client_vec {
        my_rocket = my_rocket.mount(
            client_record.url.clone(),
            FileServer::from(client_record.path.clone() + os_slash_str() + "build"),
        );
    }
    my_rocket
}
