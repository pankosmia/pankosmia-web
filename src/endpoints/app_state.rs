use crate::structs::{AppSettings, ProjectIdentifier};
use crate::utils::response::{ok_json_response, ok_ok_json_response, not_ok_json_response};
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::files::write_app_state;
use rocket::http::ContentType;
use rocket::response::status;
use rocket::http::Status;
use rocket::{get, post, State};
use serde_json::json;

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
    let new_state_json = json!(
        {
            "bcv": state.bcv.lock().unwrap().clone(),
            "current_project": project
        }
    );
    match write_app_state(state, new_state_json) {
        Ok(_) => {}
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!("Could not write app state: '{}'", &e)),
            )
        }
    }    let mut current_project_inner = state.current_project.lock().unwrap();
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
