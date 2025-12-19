use std::collections::BTreeMap;
use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::{check_path_components, check_path_string_components, os_slash_str};
use crate::utils::response::{not_ok_json_response, not_ok_bad_repo_json_response, ok_json_response};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{get, State};
use std::path::{Components, PathBuf};

/// *`GET /ingredients/raw/<repo_path>?ipath=my_burrito_path`*
///
/// Typically mounted as **`/burrito/ingredients/raw/<repo_path>?ipath=my_burrito_path`**
///
/// Returns an object containing the files directly in the specified directory as strings.
#[get("/ingredients/raw/<repo_path..>?<ipath>")]
pub async fn raw_text_ingredients(
    state: &State<AppSettings>,
    repo_path: PathBuf,
    ipath: String,
) -> status::Custom<(ContentType, String)> {
    let path_components: Components<'_> = repo_path.components();
    if check_path_components(&mut path_components.clone())
        && check_path_string_components(ipath.clone())
    {
        let dir_to_serve = state.repo_dir.lock().unwrap().clone()
            + os_slash_str()
            + &repo_path.display().to_string()
            + "/ingredients/"
            + ipath.as_str();
        let mut files = BTreeMap::new();
        let response = match std::fs::read_dir(dir_to_serve) {
            Ok(dir) => {
                let mut path;
                for entry in dir {
                    path = entry.expect("entry").path().clone();
                    if !path.is_dir() {
                        let file_path = path.clone();
                        let file_name = file_path.file_name().expect("file_name");
                        match std::fs::read_to_string(&path) {
                            Ok(fc) => {
                                files.insert(format!("{}", file_name.to_str().unwrap()), fc);
                            },
                            Err(e) => return not_ok_json_response(
                                    Status::BadRequest,
                                make_bad_json_data_response(
                                    format!("could not read ingredient file {:?}: {}", file_name, e).to_string(),
                                ),
                            )
                        };
                    }
                }
                ok_json_response(
                    serde_json::to_string(&files).expect("map to string")
                )
            }
            Err(e) => not_ok_json_response(
                Status::BadRequest,
                make_bad_json_data_response(
                    format!("could not read ingredient directory: {}", e).to_string(),
                ),
            ),
        };
        response
    } else {
        not_ok_bad_repo_json_response()
    }
}
