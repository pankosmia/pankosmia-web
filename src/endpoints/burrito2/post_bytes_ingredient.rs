use rocket::fs::TempFile;
use rocket::{post, State};
use rocket::form::{Form, FromForm};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use std::path::{Components, PathBuf};
use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::{check_path_components, check_path_string_components, os_slash_str};
use crate::utils::response::{not_ok_bad_repo_json_response, not_ok_json_response, ok_ok_json_response};

#[derive(FromForm)]
pub struct Upload<'f> {
    file: TempFile<'f>
}

/// *`POST /ingredient/bytes/<repo_path>?ipath=my_burrito_path`*
///
/// Typically mounted as **`/burrito/ingredient/bytes/<repo_path>?ipath=my_burrito_path`**
///
/// Writes a document, where the document is provided as a file upload.
#[post("/ingredient/bytes/<repo_path..>?<ipath>", format = "multipart/form-data", data = "<form>")]
pub async fn post_bytes_ingredient(state: &State<AppSettings>,
                                   repo_path: PathBuf,
                                   ipath: String,
                                   mut form: Form<Upload<'_>>, ) -> status::Custom<(ContentType, String)> {
    let path_components: Components<'_> = repo_path.components();
    let full_repo_path = state.repo_dir.lock().unwrap().clone()
        + os_slash_str()
        + &repo_path.display().to_string();
    if check_path_components(&mut path_components.clone())
        && check_path_string_components(ipath.clone())
        && std::fs::metadata(&full_repo_path).is_ok()
    {
        let destination = full_repo_path
            + "/ingredients/"
            + ipath.clone().as_str();
        let mut destination_steps: Vec<_> = destination.split("/").collect();
        destination_steps.pop().unwrap();
        let destination_steps_array = destination_steps
            .iter()
            .map(|e| format!("{:?}", e).replace("\"", ""))
            .collect::<Vec<String>>();
        let destination_parent = destination_steps_array.join("/");
        if !std::path::Path::new(&destination_parent).exists() {
            match std::fs::create_dir_all(destination_parent) {
                Ok(_) => (),
                Err(e) => {
                    return not_ok_json_response(
                        Status::InternalServerError,
                        make_bad_json_data_response(format!(
                            "Could not create local content directories: {}",
                            e
                        )),
                    )
                }
            }
        }
        match form.file.persist_to(destination).await {
            Ok(_) => ok_ok_json_response(),
            Err(e) => not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!("Could not write: {}", e)),
            )
        }
    } else {
        not_ok_bad_repo_json_response()
    }
}