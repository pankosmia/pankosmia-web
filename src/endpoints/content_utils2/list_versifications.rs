use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::os_slash_str;
use crate::utils::response::{not_ok_json_response, ok_json_response};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{get, State};

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
    let versification_paths = match std::fs::read_dir(&versification_dir) {
        Ok(paths) => paths,
        Err(err) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!(
                    "Could not read directory {} : {}",
                    versification_dir, err
                )),
            )
        }
    };
    let mut versifications: Vec<String> = Vec::new();
    for versification_path in versification_paths {
        let versification_path_ob = versification_path.unwrap().path();
        let versification_filename = versification_path_ob.file_name().unwrap();
        versifications.push(
            versification_filename
                .to_str()
                .unwrap()
                .to_string()
                .split(".")
                .next()
                .unwrap()
                .to_string(),
        );
    }
    let content_json_string = match serde_json::to_string_pretty(&versifications) {
        Ok(json_string) => json_string,
        Err(err) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!(
                    "Could not turn versifications array into JSON string: {}",
                    err
                )),
            )
        }
    };
    ok_json_response(content_json_string)
}
