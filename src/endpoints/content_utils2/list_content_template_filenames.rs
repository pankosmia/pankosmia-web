use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::os_slash_str;
use crate::utils::response::{not_ok_json_response, ok_json_response};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{get, State};

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
    let filename_paths = match std::fs::read_dir(&template_dir) {
        Ok(paths) => paths,
        Err(err) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!(
                    "Could not read directory {} : {}",
                    template_dir, err
                )),
            )
        }
    };
    let mut filenames: Vec<String> = Vec::new();
    for filename_path in filename_paths {
        let filename_path_ob = filename_path.unwrap().path();
        let filename_filename = filename_path_ob.file_name().unwrap();
        filenames.push(filename_filename.to_str().unwrap().to_string());
    }
    let content_json_string = match serde_json::to_string_pretty(&filenames) {
        Ok(json_string) => json_string,
        Err(err) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!(
                    "Could not turn filenames array into JSON string: {}",
                    err
                )),
            )
        }
    };
    ok_json_response(content_json_string)
}
