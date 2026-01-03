use crate::structs::AppSettings;
use crate::structs::BytesOrError;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::{check_path_components, check_dir_path_string_components, os_slash_str};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{get, State};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Components, PathBuf};
use tempfile::NamedTempFile;
use walkdir::WalkDir;
use zip::{write::SimpleFileOptions, ZipWriter};

/// *`GET /ingredient/zipped/<repo_path>?ipath=my_burrito_path`*
///
/// Typically mounted as **`/burrito/ingredient/zipped/<repo_path>?ipath=my_burrito_path`**
///
/// Returns a zip of ingredients under the provided path.
#[get("/ingredient/zipped/<repo_path..>?<ipath>")]
pub async fn raw_zipped_ingredient(
    state: &State<AppSettings>,
    repo_path: PathBuf,
    ipath: String,
) -> status::Custom<(ContentType, BytesOrError)> {
    let path_components: Components<'_> = repo_path.components();
    if check_path_components(&mut path_components.clone())
        && check_dir_path_string_components(ipath.clone())
    {
        // Find directory
        let path_to_serve = state.repo_dir.lock().unwrap().clone()
            + os_slash_str()
            + &repo_path.display().to_string()
            + "/ingredients/"
            + ipath.as_str();
        if !std::path::Path::new(&path_to_serve).is_dir() {
            return status::Custom(
                Status::BadRequest,
                (
                    ContentType::JSON,
                    BytesOrError::Error(
                        make_bad_json_data_response(
                            format!("could not locate ingredient directory").to_string(),
                        ),
                    ),
                ),
            );
        }
        // Iterate over ingredients, writing zip to temp file on the way
        let ingredient_walkdir = WalkDir::new(&path_to_serve);
        let prefix = std::path::Path::new(&path_to_serve);
        let ingredient_iterator = ingredient_walkdir.into_iter();
        let temp_zip_path = NamedTempFile::new().expect("tempfile");
        let mut zip = ZipWriter::new(&temp_zip_path);
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o755);
        let mut buffer = Vec::new();
        for entry_result in ingredient_iterator {
            let entry = entry_result.expect("entry");
            let path = entry.path();
            let name = path.strip_prefix(prefix).expect("strip prefix");
            let path_as_string = match name.to_str().map(str::to_owned) {
                Some(p) => p,
                None => continue,
            };
            if path.is_file() {
                // println!("file '{}'", path_as_string);
                zip.start_file(path_as_string, options).expect("start file");
                let mut f = File::open(path).expect("open file");
                f.read_to_end(&mut buffer).expect("read to end");
                zip.write_all(&buffer).expect("write");
                buffer.clear();
            } else if !name.as_os_str().is_empty() {
                // println!("dir '{}'", path_as_string);
                zip.add_directory(path_as_string, options)
                    .expect("add directory");
            }
        }
        zip.finish().expect("finish");
        // Serve temp file
        match std::fs::read(&temp_zip_path) {
            Ok(b) => status::Custom(
                Status::Ok,
                (
                    ContentType::ZIP,
                    BytesOrError::Bytes(b),
                ),
            ),
            Err(e) => status::Custom(
                Status::InternalServerError,
                (
                    ContentType::JSON,
                    BytesOrError::Error(make_bad_json_data_response(format!("Could not read zip: {}", e))),
                ),
            )
        }
    } else {
        status::Custom(
            Status::BadRequest,
            (
                ContentType::JSON,
                BytesOrError::Error(make_bad_json_data_response("bad repo path".to_string())),
            ),
        )
    }
}
