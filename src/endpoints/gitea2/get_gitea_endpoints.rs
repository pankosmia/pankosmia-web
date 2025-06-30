use rocket::{get, State};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use crate::structs::AppSettings;

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
    status::Custom(
        Status::Ok,
        (ContentType::JSON, serde_json::to_string(&state.gitea_endpoints).unwrap()),
    )
}