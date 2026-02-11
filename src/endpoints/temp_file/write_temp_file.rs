use crate::structs::{AppSettings, Upload};
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::os_slash_str;
use crate::utils::response::{not_ok_json_response, ok_json_response};
use rocket::form::Form;
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{post, State};
use serde_json::json;
use uuid::Uuid;

/// *`POST /bytes`*
///
/// Typically mounted as **`/temp/bytes`**
///
/// Writes a temporary document, where the document is provided as a file upload.
/// Returns the arbitrary id of the document

#[post("/bytes", format = "multipart/form-data", data = "<form>")]
pub async fn write_temp_file(
    state: &State<AppSettings>,
    mut form: Form<Upload<'_>>,
) -> status::Custom<(ContentType, String)> {
    let temp_id = Uuid::new_v4().to_string();
    let destination = format!(
        "{}{}temp{}{}",
        state.working_dir.clone(),
        os_slash_str(),
        os_slash_str(),
        &temp_id
    );
    // Move uploaded file to specified location
    let payload = json!({"uuid": temp_id});
    match form.file.move_copy_to(destination).await {
        Ok(_) => ok_json_response(serde_json::to_string(&payload).unwrap()),
        Err(e) => not_ok_json_response(
            Status::InternalServerError,
            make_bad_json_data_response(format!("Could not write: {}", e)),
        ),
    }
}
