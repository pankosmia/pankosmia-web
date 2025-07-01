use rocket::{get, State};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use crate::structs::AppSettings;
use crate::utils::paths::os_slash_str;

/// *`GET /template-filenames/<template>`*
///
/// Typically mounted as **`/content-utils/template-filenames/<templates>`**
///
/// Returns a JSON array of local content files for a given content template.
///
/// `["book.usfm", "metadata.json"]`
#[get("/template-filenames/<template>")]
pub fn list_content_template_filenames(
    state: &State<AppSettings>,
    template: String,
) -> status::Custom<(ContentType, String)> {
    let root_path = state.app_resources_dir.clone();
    let template_dir = format!(
        "{}{}{}{}{}{}{}",
        root_path,
        os_slash_str(),
        "templates",
        os_slash_str(),
        "content_templates",
        os_slash_str(),
        template.clone()
    );
    let filename_paths = std::fs::read_dir(template_dir).unwrap();
    let mut filenames: Vec<String> = Vec::new();
    for filename_path in filename_paths {
        let filename_path_ob = filename_path.unwrap().path();
        let filename_filename = filename_path_ob.file_name().unwrap();
        filenames.push(filename_filename.to_str().unwrap().to_string());
    }
    let content_json_string = serde_json::to_string_pretty(&filenames).unwrap();
    status::Custom(
        Status::Ok,
        (
            ContentType::JSON,
            content_json_string,
        ),
    )
}