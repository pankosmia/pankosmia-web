use crate::structs::{AppSettings, Client};
use crate::utils::client::{public_serialize_clients, Clients};
use crate::utils::json_responses::{make_bad_json_data_response, make_good_json_data_response};
use crate::utils::paths::{client_settings_dir_path, os_slash_str};
use crate::utils::response::{not_ok_json_response, ok_json_response};
use rocket::http::{ContentType, Status};
use rocket::response::{status, Redirect};
use rocket::serde::json::Json;
use rocket::{get, post, State};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;

/// *`GET /list-clients`*
///
/// Typically mounted as **`/list-clients`**
///
/// Returns a JSON array of clients.
///
/// ```text
/// [
///   {
///     "id": "core-dashboard",
///     "requires": {
///       "debug": false,
///       "net": false
///     },
///     "exclude_from_menu": false,
///     "exclude_from_dashboard": false,
///     "url": "/clients/main"
///   },
///   ...
/// ]
/// ```
#[get("/list-clients")]
pub fn list_clients(clients: &State<Clients>) -> status::Custom<(ContentType, String)> {
    let client_vec = public_serialize_clients(clients.lock().unwrap().clone());
    ok_json_response(serde_json::to_string(&client_vec).unwrap())
}

/// *`GET /client-interfaces`*
///
/// Typically mounted as **`/client-interfaces`**
///
/// Returns a JSON object of public URL interfaces offered by clients.
///
#[get("/client-interfaces")]
pub fn client_interfaces(clients: &State<Clients>) -> status::Custom<(ContentType, String)> {
    let clients = clients.lock().unwrap().clone();
    let mut summaries = BTreeMap::new();
    for client_record in clients {
        let mut endpoints_map = BTreeMap::new();
        // Get pankosmia_metadata json
        let client_path = client_record.path;
        let client_md_path = format!("{}{}pankosmia_metadata.json", &client_path, os_slash_str());
        let metadata_string = match std::fs::read_to_string(&client_md_path) {
            Ok(v) => v,
            Err(e) => {
                return not_ok_json_response(
                    Status::InternalServerError,
                    make_bad_json_data_response(format!(
                        "Could not load pankosmia metadata as string for {}: {}",
                        client_path, e
                    )),
                )
            }
        };
        let metadata_str = metadata_string.as_str();
        let metadata_json: Value = match serde_json::from_str(metadata_str) {
            Ok(j) => j,
            Err(e) => {
                return not_ok_json_response(
                    Status::InternalServerError,
                    make_bad_json_data_response(format!(
                        "Could not parse pankosmia metadata as json for {}: {}",
                        client_path, e
                    )),
                )
            }
        };
        // Look for id
        let id_value: Value = metadata_json["id"].clone();
        let id_str = id_value.as_str().expect("id as string");
        let id = format!("{}", &id_str);
        // Look for endpoints key
        let mut endpoints: Value = metadata_json["endpoints"].clone();
        if endpoints.is_null() {
            continue;
        }
        let endpoints_ob = endpoints.as_object_mut().expect("endpoints as object");
        for (key, value) in endpoints_ob {
            endpoints_map.insert(key.clone(), value.clone());
        }
        let summary = json!({
            "endpoints": endpoints_map
        });
        summaries.insert(id, summary);
    }
    ok_json_response(serde_json::to_string(&summaries).unwrap())
}

/// *`GET /client-config`*
///
/// Typically mounted as **`/client-config`**
///
/// Returns an object containing client config information if available
///
/// `{}`
#[get("/client-config")]
pub fn client_config(state: &State<AppSettings>) -> status::Custom<(ContentType, String)> {
    let client_config_struct = &state.client_config;
    let json_value = serde_json::to_string(client_config_struct).expect("serialize client config");
    ok_json_response(json_value)
}

#[get("/favicon.ico")]
pub(crate) async fn serve_root_favicon(state: &State<AppSettings>) -> Redirect {
    Redirect::to(format!(
        "/clients/{}/favicon.ico",
        state.product.homepage.clone()
    ))
}

#[get("/")]
pub(crate) fn redirect_root(state: &State<AppSettings>) -> Redirect {
    Redirect::to(format!("/clients/{}", state.product.homepage.clone()))
}

#[get("/clients/main")]
pub(crate) fn redirect_main(state: &State<AppSettings>) -> Redirect {
    Redirect::to(format!("/clients/{}", state.product.homepage.clone()))
}

/// Returns optional client matching a storage_id
fn find_client_by_storage_id(clients: Vec<Client>, storage_id: String) -> Option<Client> {
    let mut matching_client = None;
    for client in clients.iter() {
        if client.storage_id == Some(storage_id.clone()) {
            matching_client = Some(client.clone());
        }
    }
    matching_client
}

/// *`GET /client-settings/<storage_id>`*
///
/// Typically mounted as **`/client-settings/<storage_id>`**
///
/// Returns a JSON object of client settings via the client's storage id.
/// 
/// It is an error for the storage_id to not resolve to a client. If the client has no data an empty object is returned.
///
/// curl -X GET http://localhost:19119/api/client-settings/<storage_id>
#[get("/client-settings/<storage_id>")]
pub fn get_client_settings(
    state: &State<AppSettings>,
    clients: &State<Clients>,
    storage_id: String,
) -> status::Custom<(ContentType, String)> {
    let clients = clients.lock().unwrap().clone();
    let matching_client = find_client_by_storage_id(clients, storage_id);
    match matching_client {
        Some(c) => {
            let working_dir = state.working_dir.clone();
            let client_settings_path = format!(
                "{}{}{}.json",
                client_settings_dir_path(&working_dir),
                os_slash_str(),
                c.id.clone()
            );
            ok_json_response(fs::read_to_string(&client_settings_path).unwrap_or("{}".to_string()))
        }
        None => {
            return not_ok_json_response(
                Status::BadRequest,
                make_bad_json_data_response("No client found with this storage id".to_string()),
            )
        }
    }
}

/// *`POST /client-settings/<storage_id>`*
///
/// Typically mounted as **`/client-settings/<storage_id>`**
///
/// Sets settings JSON for a client via the storage_id. The JSON must be an object.
///
/// curl -X POST http://localhost:19119/api/client-settings/<storage_id> -H "Content-Type: application/json" -d '{"settings": {"foo": "baa"}}'
#[post("/client-settings/<storage_id>", format = "json", data = "<json_form>")]
pub async fn post_client_settings(
    state: &State<AppSettings>,
    clients: &State<Clients>,
    storage_id: String,
    json_form: Json<Value>,
) -> status::Custom<(ContentType, String)> {
    let clients = clients.lock().unwrap().clone();
    let matching_client = find_client_by_storage_id(clients, storage_id);
    match matching_client {
        Some(c) => {
            let settings_json = match json_form["settings"].as_object() {
            Some(j) => j,
                None => {
            return not_ok_json_response(
                Status::BadRequest,
                make_bad_json_data_response("JSON form does not have settings key or the settings value is not an object".to_string()),
            )
        }
            };
            let working_dir = state.working_dir.clone();
            let settings_dir = client_settings_dir_path(&working_dir);
            if !fs::exists(&settings_dir).expect("fs exists") {
                fs::create_dir_all(&settings_dir).expect("mkdir client settings");
            }
            let client_settings_path =
                format!("{}{}{}.json", &settings_dir, os_slash_str(), c.id.clone());
            let file_handle = fs::File::create(&client_settings_path).expect("create file handle");
            serde_json::to_writer_pretty(file_handle, &settings_json).expect("write json");
            ok_json_response(make_good_json_data_response("Settings written".to_string()))
        }
        None => {
            return not_ok_json_response(
                Status::BadRequest,
                make_bad_json_data_response("No client found with this storage id".to_string()),
            )
        }
    }
}
