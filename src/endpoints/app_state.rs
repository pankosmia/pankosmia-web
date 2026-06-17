use crate::structs::{AppSettings, ProjectIdentifier, SelectedWord};
use crate::utils::files::write_app_state;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::response::{not_ok_json_response, ok_json_response, ok_ok_json_response};
use rocket::http::ContentType;
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
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
            "current_project": {
                "source": source,
                "organization": organization,
                "project": project
            },
            "snippet": state.snippet.lock().unwrap().clone(),
            "word": state.word.lock().unwrap().clone(),
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
    }
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
    let new_state_json = json!(
        {
            "bcv": state.bcv.lock().unwrap().clone(),
            "current_project": null,
            "snippet": state.snippet.lock().unwrap().clone(),
            "word": state.word.lock().unwrap().clone(),
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
    };
    ok_ok_json_response()
}

/// *`POST /snippet/<snippet>`*
///
/// Typically mounted as **`/app-state/snippet/<snippet>`**
///
/// Sets selected snippet, unsets word.
#[post("/snippet/<snippet>")]
pub fn post_snippet(
    state: &State<AppSettings>,
    snippet: String,
) -> status::Custom<(ContentType, String)> {
    let mut snippet_inner = state.snippet.lock().unwrap();
    *snippet_inner = Some(snippet.clone());
    let mut word_inner = state.word.lock().unwrap();
    *word_inner = None;
    let new_state_json = json!(
        {
            "bcv": state.bcv.lock().unwrap().clone(),
            "current_project": state.current_project.lock().unwrap().clone(),
            "snippet": snippet,
            "word": null,
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
    };
    ok_ok_json_response()
}

/// *`POST /word`*
///
/// Typically mounted as **`/app-state/word`**
///
/// Sets selected word via JSON payload, unsets snippet.
#[post("/word", format = "json", data = "<json_form>")]
pub fn post_word(
    state: &State<AppSettings>,
    json_form: Json<SelectedWord>,
) -> status::Custom<(ContentType, String)> {
    if json_form.target.is_none() && json_form.source.is_none() && json_form.lemma.is_none() {
        return not_ok_json_response(
            Status::InternalServerError,
            make_bad_json_data_response(format!("At least one of target, source and lemma must be provided")),
        );
    };
    let word_json = json!({
        "target": match &json_form.target {Some(v) => Some(v), None => None},
        "source": match &json_form.source {Some(v) => Some(v), None => None},
        "lemma": match &json_form.lemma {Some(v) => Some(v), None => None},
    });
    let new_state_json = json!(
        {
            "bcv": state.bcv.lock().unwrap().clone(),
            "current_project": state.current_project.lock().unwrap().clone(),
            "snippet": null,
            "word": &word_json,
        }
    );
    let mut snippet_inner = state.snippet.lock().unwrap();
    *snippet_inner = None;
    let mut word_inner = state.word.lock().unwrap();
    *word_inner = Some(serde_json::from_value(word_json).unwrap());
    match write_app_state(state, new_state_json) {
        Ok(_) => {}
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!("Could not write app state: '{}'", &e)),
            )
        }
    };
    ok_ok_json_response()
}

/// *`POST /empty-alignment`*
///
/// Typically mounted as **`/app-state/empty_alignment`**
///
/// Unsets current word and snippet.
#[post("/empty-alignment")]
pub fn post_empty_alignment(
    state: &State<AppSettings>,
) -> status::Custom<(ContentType, String)> {
    let mut snippet_inner = state.snippet.lock().unwrap();
    *snippet_inner = None;
    let mut word_inner = state.word.lock().unwrap();
    *word_inner = None;
    let new_state_json = json!(
        {
            "bcv": state.bcv.lock().unwrap().clone(),
            "current_project": state.current_project.lock().unwrap().clone(),
            "snippet": null,
            "word": null,
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
    };
    ok_ok_json_response()
}