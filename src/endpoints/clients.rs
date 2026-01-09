use std::collections::BTreeMap;
use crate::utils::client::{public_serialize_clients, Clients};
use crate::utils::response::{not_ok_json_response, ok_json_response};
use rocket::http::{ContentType, Status};
use rocket::response::{status, Redirect};
use rocket::{get, State};
use serde_json::Value;
use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::os_slash_str;

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
/// ```text

#[get("/client-interfaces")]
pub fn client_interfaces(clients: &State<Clients>) -> status::Custom<(ContentType, String)> {
    let clients = clients.lock().unwrap().clone();
    let mut summary = BTreeMap::new();
    for client_record in clients {
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
                        client_path,
                        e
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
                        client_path,
                        e
                    )),
                )
            }
        };
        // Look for endpoints key
        let mut endpoints: Value = metadata_json["endpoints"].clone();
        if endpoints.is_null() {
            continue;
        }
        let endpoints_ob = endpoints.as_object_mut().unwrap();
        for (key, value) in endpoints_ob {
            summary.insert(key.clone(), value.clone());
        }
    };
    ok_json_response(serde_json::to_string(&summary).unwrap())
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
pub(crate) async fn serve_root_favicon() -> Redirect {
    Redirect::to("/clients/main/favicon.ico")
}

#[get("/")]
pub(crate) fn redirect_root() -> Redirect {
    Redirect::to("/clients/main")
}
