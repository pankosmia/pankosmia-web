use std::path::{Components, PathBuf};
use hallomai::transform;
use rocket::{get, post, State};
use rocket::form::Form;
use rocket::http::{ContentType, Status};
use rocket::response::status;
use serde_json::Value;
use crate::structs::{AppSettings, MetadataSummary, Upload};
use crate::utils::json_responses::{make_bad_json_data_response, make_good_json_data_response};
use crate::utils::mime::mime_types;
use crate::utils::paths::{check_path_components, check_path_string_components, os_slash_str};

/// *`GET /metadata/raw/<repo_path>`*
///
/// Typically mounted as **`/burrito/metadata/raw/<repo_path>`**
///
/// Returns the raw metadata.json file for the specified burrito, where *repo_path* is *`<server>/<org>/<repo>`* and refers to a local repo.
#[get("/metadata/raw/<repo_path..>")]
pub async fn raw_metadata(
    state: &State<AppSettings>,
    repo_path: PathBuf,
) -> status::Custom<(ContentType, String)> {
    let path_components: Components<'_> = repo_path.components();
    if check_path_components(&mut path_components.clone()) {
        let path_to_serve = state.repo_dir.lock().unwrap().clone()
            + os_slash_str()
            + &repo_path.display().to_string()
            + "/metadata.json";
        match std::fs::read_to_string(path_to_serve) {
            Ok(v) => status::Custom(Status::Ok, (ContentType::JSON, v)),
            Err(e) => status::Custom(
                Status::BadRequest,
                (
                    ContentType::JSON,
                    make_bad_json_data_response(
                        format!("could not read metadata: {}", e).to_string(),
                    ),
                ),
            ),
        }
    } else {
        status::Custom(
            Status::BadRequest,
            (
                ContentType::JSON,
                make_bad_json_data_response("bad repo path".to_string()),
            ),
        )
    }
}

/// *`GET /metadata/summary/<repo_path>`*
///
/// Typically mounted as **`/burrito/metadata/summary/<repo_path>`**
///
/// Returns a flat summary of information from the raw metadata.json file for the specified burrito, where *repo_path* is *`<server>/<org>/<repo>`* and refers to a local repo. eg, the response to `/burrito/metadata/summary/git.door43.org/BurritoTruck/fr_psle` might be
///
/// ```
/// {
///   "name": "Pain Sur Les Eaux",
///   "description": "Une traduction littéralement plus simple",
///   "flavor_type": "scripture",
///   "flavor": "textTranslation",
///   "language_code": "fr",
///   "script_direction": "ltr"
/// }
/// ```
#[get("/metadata/summary/<repo_path..>")]
pub async fn summary_metadata(
    state: &State<AppSettings>,
    repo_path: PathBuf,
) -> status::Custom<(ContentType, String)> {
    let path_components: Components<'_> = repo_path.components();
    if check_path_components(&mut path_components.clone()) {
        let path_to_serve = state.repo_dir.lock().unwrap().clone()
            + os_slash_str()
            + &repo_path.display().to_string()
            + os_slash_str()
            + "metadata.json";
        println!("{}", path_to_serve);
        let file_string = match std::fs::read_to_string(path_to_serve) {
            Ok(v) => v,
            Err(e) => {
                return status::Custom(
                    Status::BadRequest,
                    (
                        ContentType::JSON,
                        make_bad_json_data_response(
                            format!("could not read metadata: {}", e).to_string(),
                        ),
                    ),
                )
            }
        };
        let raw_metadata_struct: Value =
            match serde_json::from_str(file_string.as_str()) {
                Ok(v) => v,
                Err(e) => {
                    return status::Custom(
                        Status::BadRequest,
                        (
                            ContentType::JSON,
                            make_bad_json_data_response(
                                format!("could not parse metadata: {}", e).to_string(),
                            ),
                        ),
                    )
                }
            };
        let summary = MetadataSummary {
            name: raw_metadata_struct["identification"]["name"]["en"]
                .as_str()
                .unwrap()
                .to_string(),
            description: match raw_metadata_struct["identification"]["description"]["en"].clone() {
                Value::String(v) => v.as_str().to_string(),
                Value::Null => "".to_string(),
                _ => "?".to_string(),
            },
            flavor_type: raw_metadata_struct["type"]["flavorType"]["name"]
                .as_str()
                .unwrap()
                .to_string(),
            flavor: raw_metadata_struct["type"]["flavorType"]["flavor"]["name"]
                .as_str()
                .unwrap()
                .to_string(),
            language_code: raw_metadata_struct["languages"][0]["tag"]
                .as_str()
                .unwrap()
                .to_string(),
            script_direction: match raw_metadata_struct["languages"][0]["scriptDirection"].clone() {
                Value::String(v) => v.as_str().to_string(),
                _ => "?".to_string(),
            },
        };
        match serde_json::to_string(&summary) {
            Ok(v) => status::Custom(Status::Ok, (ContentType::JSON, v)),
            Err(e) => status::Custom(
                Status::InternalServerError,
                (
                    ContentType::JSON,
                    make_bad_json_data_response(
                        format!("could not serialize metadata: {}", e).to_string(),
                    ),
                ),
            ),
        }
    } else {
        status::Custom(
            Status::BadRequest,
            (
                ContentType::JSON,
                make_bad_json_data_response("bad repo path!".to_string()),
            ),
        )
    }
}

// INGREDIENT OPERATIONS
/// *`GET /ingredient/raw/<repo_path>?ipath=my_burrito_path`*
///
/// Typically mounted as **`/burrito/ingredient/raw/<repo_path>?ipath=my_burrito_path`**
///
/// Returns a raw resource. We try to guess the mimetype.
#[get("/ingredient/raw/<repo_path..>?<ipath>")]
pub async fn raw_ingredient(
    state: &State<AppSettings>,
    repo_path: PathBuf,
    ipath: String,
) -> status::Custom<(ContentType, String)> {
    let path_components: Components<'_> = repo_path.components();
    if check_path_components(&mut path_components.clone())
        && check_path_string_components(ipath.clone())
    {
        let path_to_serve = state.repo_dir.lock().unwrap().clone()
            + os_slash_str()
            + &repo_path.display().to_string()
            + "/ingredients/"
            + ipath.as_str();
        match std::fs::read_to_string(path_to_serve) {
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
                        v,
                    ),
                )
            }
            Err(e) => status::Custom(
                Status::BadRequest,
                (
                    ContentType::JSON,
                    make_bad_json_data_response(
                        format!("could not read ingredient content: {}", e).to_string(),
                    ),
                ),
            ),
        }
    } else {
        status::Custom(
            Status::BadRequest,
            (
                ContentType::JSON,
                make_bad_json_data_response("bad repo path".to_string()),
            ),
        )
    }
}

/// *`GET /ingredient/as-usj/<repo_path>?ipath=my_burrito_path`*
///
/// Typically mounted as **`/burrito/ingredient/as-usj/<repo_path>?ipath=my_burrito_path`**
///
/// Returns a USFM resource as USJ. Currently slow and buggy but works for typical CCBT USFM.
#[get("/ingredient/as-usj/<repo_path..>?<ipath>")]
pub async fn get_ingredient_as_usj(
    state: &State<AppSettings>,
    repo_path: PathBuf,
    ipath: String,
) -> status::Custom<(ContentType, String)> {
    let path_components: Components<'_> = repo_path.components();
    if check_path_components(&mut path_components.clone())
        && check_path_string_components(ipath.clone())
    {
        let path_to_serve = state.repo_dir.lock().unwrap().clone()
            + os_slash_str()
            + &repo_path.display().to_string()
            + "/ingredients/"
            + ipath.as_str();
        match std::fs::read_to_string(path_to_serve) {
            Ok(v) => status::Custom(
                Status::Ok,
                (
                    ContentType::JSON,
                    transform(v, "usfm".to_string(), "usj".to_string()),
                ),
            ),
            Err(e) => status::Custom(
                Status::BadRequest,
                (
                    ContentType::JSON,
                    make_bad_json_data_response(
                        format!("could not read ingredient content: {}", e).to_string(),
                    ),
                ),
            ),
        }
    } else {
        status::Custom(
            Status::BadRequest,
            (
                ContentType::JSON,
                make_bad_json_data_response("bad repo path".to_string()),
            ),
        )
    }
}

/// *`POST /ingredient/as-usj/<repo_path>?ipath=my_burrito_path`*
///
/// Typically mounted as **`/burrito/ingredient/as-usj/<repo_path>?ipath=my_burrito_path`**
///
/// Returns a USJ document as USFM, where the USJ is provided as an HTTP form file. Currently slow and buggy but works for typical CCBT USFM.
#[post(
    "/ingredient/as-usj/<repo_path..>?<ipath>",
    format = "multipart/form-data",
    data = "<form>"
)]
pub async fn post_ingredient_as_usj(
    state: &State<AppSettings>,
    repo_path: PathBuf,
    ipath: String,
    mut form: Form<Upload<'_>>,
) -> status::Custom<(ContentType, String)> {
    let path_components: Components<'_> = repo_path.components();
    let destination = state.repo_dir.lock().unwrap().clone()
        + os_slash_str()
        + &repo_path.display().to_string()
        + "/ingredients/"
        + ipath.clone().as_str();
    if check_path_components(&mut path_components.clone())
        && check_path_string_components(ipath)
        && std::fs::metadata(destination.clone()).is_ok()
    {
        let _ = form
            .file
            .persist_to(transform(
                destination,
                "usj".to_string(),
                "usfm".to_string(),
            ))
            .await;
        status::Custom(
            Status::Ok,
            (
                ContentType::JSON,
                make_good_json_data_response("ok".to_string()),
            ),
        )
    } else {
        status::Custom(
            Status::BadRequest,
            (
                ContentType::JSON,
                make_bad_json_data_response("bad repo path".to_string()),
            ),
        )
    }
}

/// *`GET /ingredient/prettified/<repo_path>?ipath=my_burrito_path`*
///
/// Typically mounted as **`/burrito/ingredient/prettified/<repo_path>?ipath=my_burrito_path`**
///
/// Returns a text-like resource as a web page.
#[get("/ingredient/prettified/<repo_path..>?<ipath>")]
pub async fn get_ingredient_prettified(
    state: &State<AppSettings>,
    repo_path: PathBuf,
    ipath: String,
) -> status::Custom<(ContentType, String)> {
    let path_components: Components<'_> = repo_path.components();
    if check_path_components(&mut path_components.clone())
        && check_path_string_components(ipath.clone())
    {
        let path_to_serve = state.repo_dir.lock().unwrap().clone()
            + os_slash_str()
            + &repo_path.display().to_string()
            + "/ingredients/"
            + ipath.as_str();
        let file_string = match std::fs::read_to_string(path_to_serve) {
            Ok(v) => v,
            Err(e) => {
                return status::Custom(
                    Status::BadRequest,
                    (
                        ContentType::JSON,
                        make_bad_json_data_response(
                            format!("could not read ingredient content: {}", e).to_string(),
                        ),
                    ),
                )
            }
        };
        status::Custom(
            Status::Ok,
            (
                ContentType::HTML,
                format!(
                    r#"
                <html>
                <head>
                <title>Prettified</title>
                <link rel="stylesheet" href="/webfonts/_webfonts.css">
                </head>
                <body>
                <pre>
                {}
                </pre>
                </body>
                </html>
                "#,
                    file_string
                ),
            ),
        )
    } else {
        status::Custom(
            Status::BadRequest,
            (
                ContentType::JSON,
                make_bad_json_data_response("bad repo path".to_string()),
            ),
        )
    }
}
