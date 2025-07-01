use rocket::{get, State};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use crate::structs::AppSettings;
use crate::utils::paths::os_slash_str;

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
    let templates_dir = format!("{}{}{}{}{}", root_path, os_slash_str(), "templates", os_slash_str(), "content_templates");
    let template_paths = std::fs::read_dir(templates_dir).unwrap();
    let mut templates: Vec<String> = Vec::new();
    for template_path in template_paths {
        let template_path_ob = template_path.unwrap().path();
        let template_filename = template_path_ob.file_name().unwrap();
        templates.push(template_filename.to_str().unwrap().to_string().split(".").next().unwrap().to_string());
    }
    let content_json_string = serde_json::to_string_pretty(&templates).unwrap();
    status::Custom(
        Status::Ok,
        (
            ContentType::JSON,
            content_json_string,
        ),
    )
}