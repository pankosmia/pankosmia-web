use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::os_slash_str;
use crate::utils::response::{not_ok_json_response, ok_json_response};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{get, State};

/// *`GET /templates`*
///
/// Typically mounted as **`/content-utils/templates`**
///
/// Returns a JSON array of local content template names.
///
/// `["text_translation"]`
#[get("/templates")]
pub fn list_content_templates(state: &State<AppSettings>) -> status::Custom<(ContentType, String)> {
    let root_path = state.app_resources_dir.clone();
    let templates_dir = format!(
        "{}{}{}{}{}",
        root_path,
        os_slash_str(),
        "templates",
        os_slash_str(),
        "content_templates"
    );
    let template_paths = match std::fs::read_dir(&templates_dir) {
        Ok(paths) => paths,
        Err(err) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!(
                    "Could not read directory {} : {}",
                    templates_dir, err
                )),
            )
        }
    };
    let mut templates: Vec<String> = Vec::new();
    for template_path in template_paths {
        let template_path_ob = template_path.unwrap().path();
        let template_filename = template_path_ob.file_name().unwrap();
        templates.push(
            template_filename
                .to_str()
                .unwrap()
                .to_string()
                .split(".")
                .next()
                .unwrap()
                .to_string(),
        );
    }
    let content_json_string = match serde_json::to_string_pretty(&templates) {
        Ok(json_string) => json_string,
        Err(err) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!(
                    "Could not turn templates array into JSON string: {}",
                    err
                )),
            )
        }
    };
    ok_json_response(content_json_string)
}
