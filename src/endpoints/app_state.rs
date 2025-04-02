use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{get, post, State};
use crate::structs::{AppSettings, ProjectIdentifier};
use crate::utils::json_responses::make_good_json_data_response;

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
        Some(p) => status::Custom(
            Status::Ok,
            (
                ContentType::JSON,
                serde_json::to_string_pretty(&p).unwrap(),
            )
        ),
        None => status::Custom(
            Status::Ok,
            (
                ContentType::JSON,
                "null".to_string()
            )
        ),
    }
}

/// *`POST /current-project/<source>/<organization>/<project>`*
///
/// Typically mounted as **`/app-state/current-project/<source>/<organization>/<project>`**
///
/// Sets current project.
#[post("/current-project/<source>/<organization>/<project>")]
pub fn post_current_project(state: &State<AppSettings>, source: &str, organization: &str, project: &str) -> status::Custom<(ContentType, String)> {
    let mut current_project_inner = state.current_project.lock().unwrap();
    *current_project_inner = Some(ProjectIdentifier{source: source.to_string(), organization: organization.to_string(), project: project.to_string()});
    status::Custom(
        Status::Ok,
        (
            ContentType::JSON,
            make_good_json_data_response("Ok".to_string()),
        )
    )
}

/// *`POST /empty-current-project`*
///
/// Typically mounted as **`/app-state/empty_current-project`**
///
/// Unsets current project.
#[post("/current-project")]
pub fn post_empty_current_project(state: &State<AppSettings>) -> status::Custom<(ContentType, String)> {
    let mut current_project_inner = state.current_project.lock().unwrap();
    *current_project_inner = None;
    status::Custom(
        Status::Ok,
        (
            ContentType::JSON,
            make_good_json_data_response("Ok".to_string()),
        )
    )
}