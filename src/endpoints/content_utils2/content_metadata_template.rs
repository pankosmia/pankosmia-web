use rocket::{get, State};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::os_slash_str;

/// *`GET /metadata-template/<template_name>`*
///
/// Typically mounted as **`/content-utils/metadata-template/<template_name>`**
///
/// Returns a metadata content template as JSON
#[get("/metadata-template/<template_name>")]
pub async fn content_metadata_template(
    state: &State<AppSettings>,
    template_name: String,
) -> status::Custom<(ContentType, String)> {
    let path_to_serve = format!(
        "{}{}{}{}{}{}{}{}{}",
        &state.app_resources_dir,
        os_slash_str(),
        "templates",
        os_slash_str(),
        "content_templates",
        os_slash_str(),
        template_name.clone(),
        os_slash_str(),
        "metadata.json"
    );

    match std::fs::read_to_string(path_to_serve) {
        Ok(v) => {
            status::Custom(
                Status::Ok,
                (
                    ContentType::JSON,
                    v,
                ),
            )
        }
        Err(e) => status::Custom(
            Status::BadRequest,
            (
                ContentType::JSON,
                make_bad_json_data_response(
                    format!("could not read content metadata template '{}': {}", template_name, e).to_string(),
                ),
            ),
        ),
    }
}