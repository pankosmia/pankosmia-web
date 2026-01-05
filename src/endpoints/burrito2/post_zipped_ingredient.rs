use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::{check_path_components, check_path_string_components, os_slash_str};
use crate::utils::response::{
    not_ok_bad_repo_json_response, not_ok_json_response, ok_ok_json_response,
};
use crate::utils::burrito::{destination_parent};
use rocket::form::{Form, FromForm};
use rocket::fs::TempFile;
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{post, State};
use std::path::{Components, PathBuf};
use tempfile::NamedTempFile;
use crate::utils::zip::unpack_zip_file;

#[derive(FromForm)]
pub struct Upload<'f> {
    file: TempFile<'f>,
}

/// *`POST /ingredient/zipped/<repo_path>?ipath=my_burrito_path`*
///
/// Typically mounted as **`/burrito/ingredient/zipped/<repo_path>?ipath=my_burrito_path`**
///
/// Writes files or directories provided as a zip file.
#[post(
    "/ingredient/zipped/<repo_path..>?<ipath>",
    format = "multipart/form-data",
    data = "<form>"
)]
pub async fn post_zipped_ingredient(
    state: &State<AppSettings>,
    repo_path: PathBuf,
    ipath: String,
    mut form: Form<Upload<'_>>,
) -> status::Custom<(ContentType, String)> {
    let path_components: Components<'_> = repo_path.components();
    let full_repo_path =
        format!("{}{}{}", state.repo_dir.lock().unwrap(), os_slash_str(), &repo_path.display().to_string());
    if check_path_components(&mut path_components.clone())
        && check_path_string_components(ipath.clone())
        && std::fs::metadata(&full_repo_path).is_ok()
    {
        let destination = format!("{}{}ingredients{}{}", &full_repo_path, os_slash_str(), os_slash_str(), &ipath);
        let destination_parent = destination_parent(destination.clone());
        // Make subdirs if necessary
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

        // Copy upload to temp file we manage
        let file_path = NamedTempFile::new().expect("tempfile");
        form.file.move_copy_to(&file_path).await.expect("copy zip");

        // Unpack zip
        match unpack_zip_file(file_path, destination).await {
            Ok(_) => ok_ok_json_response(),
            Err(e) => not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!(
                    "Could not unpack zip archive: {}",
                    e
                )),
            )
        }
    } else {
        not_ok_bad_repo_json_response()
    }
}
