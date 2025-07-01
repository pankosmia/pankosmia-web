use rocket::{get, State};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::os_slash_str;

/// *`GET /template/<template_name>/<filename>`*
///
/// Typically mounted as **`/content-utils/template/<template_name>/<filename>`**
///
/// Returns a content template of a particular type as plain text. The filename includes the suffix.
#[get("/template/<template_name>/<filename>")]
pub async fn content_template(
    state: &State<AppSettings>,
    template_name: String,
    filename: String
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
        filename.clone()
    );

    match std::fs::read_to_string(path_to_serve) {
        Ok(v) => {
            status::Custom(
                Status::Ok,
                (
                    ContentType::Plain,
                    v,
                ),
            )
        }
        Err(e) => status::Custom(
            Status::BadRequest,
            (
                ContentType::JSON,
                make_bad_json_data_response(
                    format!(
                        "could not read file {} for content template '{}': {}",
                        filename,
                        template_name,
                        e
                    ).to_string(),
                ),
            ),
        ),
    }
}