use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::{check_path_components, os_slash_str};
use crate::utils::response::{
    not_ok_bad_repo_json_response, not_ok_json_response, ok_ok_json_response,
};
use crate::utils::zip::unpack_zip_file;
use rocket::form::{Form, FromForm};
use rocket::fs::TempFile;
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{post, State};
use std::fs::File;
use std::path::{Components, Path, PathBuf};
use tempfile::NamedTempFile;
use zip::ZipArchive;

#[derive(FromForm)]
pub struct Upload<'f> {
    file: TempFile<'f>,
}

/// Returns true if the zip looks a bit like a burrito
fn check_burrito_zip(path: &NamedTempFile) -> bool {
    let zip_file = File::open(path).expect("open zip archive file to check");
    let mut archive = ZipArchive::new(zip_file).expect("new archive to check");
    // Iterate over archive files, looking for metadata and ingredients
    let mut metadata_found = false;
    let mut ingredients_found = false;
    for i in 0..archive.len() {
        let file = archive.by_index(i).expect("file from zip to check");
        let out_path = match file.enclosed_name() {
            Some(p) => p,
            None => continue,
        };
        let out_path_string = format!("{:?}", out_path);
        if file.is_file() {
            if out_path_string == "\"metadata.json\"" {
                metadata_found = true;
            }
        } else {
            if out_path_string == "\"ingredients/\"" {
                ingredients_found = true;
            }
        }
    }
    metadata_found && ingredients_found
}

/// *`POST /zipped/<repo_path>`*
///
/// Typically mounted as **`/burrito/zipped/<repo_path>`**
///
/// Writes a new repo from a zip.
#[post(
    "/zipped/<repo_path..>",
    format = "multipart/form-data",
    data = "<form>"
)]
pub async fn post_zipped_repo(
    state: &State<AppSettings>,
    repo_path: PathBuf,
    mut form: Form<Upload<'_>>,
) -> status::Custom<(ContentType, String)> {
    let path_components: Components<'_> = repo_path.components();
    let full_repo_path = format!(
        "{}{}{}",
        state.repo_dir.lock().unwrap(),
        os_slash_str(),
        &repo_path.display().to_string()
    );
    if check_path_components(&mut path_components.clone()) {
        if Path::new(&full_repo_path).exists() {
            return not_ok_json_response(
                Status::BadRequest,
                make_bad_json_data_response(format!("Repo already exists")),
            );
        }
        // Copy upload to temp file we manage
        let file_path = NamedTempFile::new().expect("tempfile");
        form.file.move_copy_to(&file_path).await.expect("copy zip");

        // Check burrito
        if !check_burrito_zip(&file_path) {
            return not_ok_json_response(
                Status::BadRequest,
                make_bad_json_data_response("Zip does not look like a burrito".to_string()),
            )
        }

        match std::fs::create_dir_all(&full_repo_path) {
            Ok(_) => (),
            Err(e) => {
                return not_ok_json_response(
                    Status::InternalServerError,
                    make_bad_json_data_response(format!("Could not create repo dir: {}", e)),
                )
            }
        }

        // Unpack zip
        match unpack_zip_file(file_path, full_repo_path).await {
            Ok(_) => ok_ok_json_response(),
            Err(e) => not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!("Could not write zip archive: {}", e)),
            ),
        }
    } else {
        not_ok_bad_repo_json_response()
    }
}
