use crate::structs::AppSettings;
use crate::utils::response::ok_json_response;
use rocket::http::{ContentType};
use rocket::response::status;
use rocket::{get, State};

/// *```GET /endpoints```*
///
/// Typically mounted as **`/gitea/endpoints`**
///
/// Returns an object containing gitea gateway keys and urls.
///
/// ```text
/// {"xenizo_syllogos":"http://xenizo.fr:8089"}
/// ```
#[get("/endpoints")]
pub fn get_gitea_endpoints(state: &State<AppSettings>) -> status::Custom<(ContentType, String)> {
    ok_json_response(serde_json::to_string(&state.gitea_endpoints).unwrap())
}
