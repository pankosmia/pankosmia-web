use rocket::{get, State};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use crate::structs::AppSettings;
use crate::utils::paths::os_slash_str;

/// *`GET /versifications`*
///
/// Typically mounted as **`/content-utils/versifications`**
///
/// Returns a JSON array of versification schemes.
///
/// `["ENG", "ORG", "LXX"]`
#[get("/versifications")]
pub fn list_versifications(state: &State<AppSettings>) -> status::Custom<(ContentType, String)> {
    let root_path = state.app_resources_dir.clone();
    let versification_dir = format!(
        "{}{}{}{}{}{}{}",
        root_path,
        os_slash_str(),
        "templates",
        os_slash_str(),
        "content_templates",
        os_slash_str(),
        "vrs"
    );
    let versification_paths = std::fs::read_dir(versification_dir).unwrap();
    let mut versifications: Vec<String> = Vec::new();
    for versification_path in versification_paths {
        let versification_path_ob = versification_path.unwrap().path();
        let versification_filename = versification_path_ob.file_name().unwrap();
        versifications.push(versification_filename.to_str().unwrap().to_string().split(".").next().unwrap().to_string());
    }
    let content_json_string = serde_json::to_string_pretty(&versifications).unwrap();
    status::Custom(
        Status::Ok,
        (
            ContentType::JSON,
            content_json_string,
        ),
    )
}