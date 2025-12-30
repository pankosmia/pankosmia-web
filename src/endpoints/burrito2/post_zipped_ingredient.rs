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
use zip::ZipArchive;
use tempfile::NamedTempFile;

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

        // Make zip struct
        let file_path = NamedTempFile::new().expect("tempfile");
        form.file.move_copy_to(&file_path).await.expect("copy zip");
        let zip_file = std::fs::File::open(file_path).expect("open zip copy");
        let mut archive = match ZipArchive::new(zip_file) {
            Ok(a) => a,
            Err(e) => return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!("Could not create zip struct: {}", e)),
            ),
        };
        // Iterate over archive files, ignoring bad ones
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).expect("file from zip");
            let out_path = match file.enclosed_name() {
                Some(p) => p,
                None => continue
            };
            if !file.is_file() {
                continue;
            }
            let full_out_path = format!(
                "{}{}{}",
                destination,
                os_slash_str(),
                out_path.display()
            );
            let out_path_parent = std::path::Path::new(&full_out_path).parent().expect("parent");
            if !out_path_parent.exists() {
                std::fs::create_dir_all(&out_path_parent).expect("create all dirs");
            }
            let mut out_file = std::fs::File::create(&full_out_path).expect("create");
            std::io::copy(&mut file, &mut out_file).expect("write");
        }
        ok_ok_json_response()
    } else {
        not_ok_bad_repo_json_response()
    }
}
