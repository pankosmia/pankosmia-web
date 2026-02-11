use crate::structs::{AppSettings, BytesOrError};
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::os_slash_str;
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{get, State};

/// *`GET /bytes/<temp_id>`*
///
/// Typically mounted as **`/temp/bytes/<temp_id>`**
///
/// Returns a raw binary temp file.
#[get("/bytes/<temp_id>")]
pub async fn read_temp_file(
    state: &State<AppSettings>,
    temp_id: String,
) -> status::Custom<(ContentType, BytesOrError)> {
    let path_to_serve = format!(
        "{}{}temp{}{}",
        state.working_dir.clone(),
        os_slash_str(),
        os_slash_str(),
        &temp_id
    );
    match std::fs::read(path_to_serve) {
        Ok(v) => status::Custom(
            Status::Ok,
            (
                ContentType::new("application", "octet-stream"),
                BytesOrError::Bytes(v),
            ),
        ),
        Err(e) => status::Custom(
            Status::BadRequest,
            (
                ContentType::JSON,
                BytesOrError::Error(make_bad_json_data_response(
                    format!("could not read temp file: {}", e).to_string(),
                )),
            ),
        ),
    }
}
