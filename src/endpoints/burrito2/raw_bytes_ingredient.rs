use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::mime::mime_types;
use crate::utils::paths::{check_path_components, check_path_string_components, os_slash_str};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{get, Responder, State};
use std::path::{Components, PathBuf};

#[derive(Responder)]
pub enum BytesOrError {
    Error(String),
    Bytes(Vec<u8>),
}

/// *`GET /ingredient/bytes/<repo_path>?ipath=my_burrito_path`*
///
/// Typically mounted as **`/burrito/ingredient/bytes/<repo_path>?ipath=my_burrito_path`**
///
/// Returns a raw binary resource. We try to guess the mimetype.
#[get("/ingredient/bytes/<repo_path..>?<ipath>")]
pub async fn raw_bytes_ingredient(
    state: &State<AppSettings>,
    repo_path: PathBuf,
    ipath: String,
) -> status::Custom<(ContentType, BytesOrError)> {
    let path_components: Components<'_> = repo_path.components();
    if check_path_components(&mut path_components.clone())
        && check_path_string_components(ipath.clone())
    {
        let path_to_serve = state.repo_dir.lock().unwrap().clone()
            + os_slash_str()
            + &repo_path.display().to_string()
            + "/ingredients/"
            + ipath.as_str();
        match std::fs::read(path_to_serve) {
            Ok(v) => {
                let mut split_ipath = ipath.split(".").clone();
                let mut suffix = "unknown";
                if let Some(_) = split_ipath.next() {
                    if let Some(second) = split_ipath.next() {
                        suffix = second;
                    }
                }
                status::Custom(
                    Status::Ok,
                    (
                        match mime_types().get(suffix) {
                            Some(t) => t.clone(),
                            None => ContentType::new("application", "octet-stream"),
                        },
                        BytesOrError::Bytes(v),
                    ),
                )
            }
            Err(e) => status::Custom(
                Status::BadRequest,
                (
                    ContentType::JSON,
                    BytesOrError::Error(make_bad_json_data_response(
                        format!("could not read ingredient content: {}", e).to_string(),
                    )),
                ),
            ),
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
