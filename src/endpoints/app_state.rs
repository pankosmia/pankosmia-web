use crate::structs::{AppSettings, ProjectIdentifier};
use crate::utils::response::{ok_json_response, ok_ok_json_response};
use rocket::http::ContentType;
use rocket::response::status;
use rocket::{get, post, State};

/// *`GET /current-project`*
///
/// Typically mounted as **`/app-state/current-project`**
///
/// Returns a JSON description of the current project, or null.
///
/// ```text
/// {
///   "source": "_local",
///   "organization": "\"_local\"",
///   "project": "\"my_project\""
/// }
/// ```
#[get("/current-project")]
pub fn get_current_project(state: &State<AppSettings>) -> status::Custom<(ContentType, String)> {
    let current_project_inner = state.current_project.lock().unwrap().clone();
    match current_project_inner {
        Some(p) => ok_json_response(serde_json::to_string_pretty(&p).unwrap()),
        None => ok_json_response("null".to_string()),
    }
}

/// *`POST /current-project/<source>/<organization>/<project>`*
///
/// Typically mounted as **`/app-state/current-project/<source>/<organization>/<project>`**
///
/// Sets current project.
#[post("/current-project/<source>/<organization>/<project>")]
pub fn post_current_project(
    state: &State<AppSettings>,
    source: &str,
    organization: &str,
    project: &str,
) -> status::Custom<(ContentType, String)> {
    let mut current_project_inner = state.current_project.lock().unwrap();
    *current_project_inner = Some(ProjectIdentifier {
        source: source.to_string(),
        organization: organization.to_string(),
        project: project.to_string(),
    });
    ok_ok_json_response()
}

/// *`POST /empty-current-project`*
///
/// Typically mounted as **`/app-state/empty_current-project`**
///
/// Unsets current project.
#[post("/current-project")]
pub fn post_empty_current_project(
    state: &State<AppSettings>,
) -> status::Custom<(ContentType, String)> {
    let mut current_project_inner = state.current_project.lock().unwrap();
    *current_project_inner = None;
    ok_ok_json_response()
}
