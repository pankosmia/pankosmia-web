use rocket::{get, State};
use rocket::http::{ContentType, Status};
use rocket::response::{status, Redirect};
use crate::utils::client::{public_serialize_clients, Clients};

/// *`GET /list-clients`*
///
/// Typically mounted as **`/list-clients`**
///
/// Returns a JSON array of clients.
///
/// ```
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
    status::Custom(
        Status::Ok,
        (
            ContentType::JSON,
            serde_json::to_string(&client_vec).unwrap(),
        ),
    )
}

#[get("/favicon.ico")]
pub(crate) async fn serve_root_favicon() -> Redirect {
    Redirect::to("/clients/main/favicon.ico")
}

#[get("/")]
pub(crate) fn redirect_root() -> Redirect {
    Redirect::to("/clients/main")
}

