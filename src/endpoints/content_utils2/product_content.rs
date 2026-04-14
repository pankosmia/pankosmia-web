use crate::structs::{AppSettings, BytesOrError};
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::mime::mime_types;
use crate::utils::paths::{check_path_string_components, os_slash_str};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{get, State};

/// *`GET /product?<resource_path>`*
///
/// Typically mounted as **`/content-utils/product?<resource_path>`**
///
/// Returns a product-specific resource
/// Note that there should only be one set of product content at any one time.
#[get("/product?<resource_path>")]
pub async fn product_content_catalog(
    state: &State<AppSettings>,
    resource_path: String,
) -> status::Custom<(ContentType, BytesOrError)> {
    if check_path_string_components(resource_path.clone()) {
        let path_to_serve = format!(
            "{}{}{}{}{}{}{}{}{}",
            &state.app_resources_dir,
            os_slash_str(),
            "app_resources",
            os_slash_str(),
            "product",
            os_slash_str(),
            "product_resources",
            os_slash_str(),
            &resource_path
        );
        match std::fs::read(&path_to_serve) {
            Ok(v) => {
                let mut split_resource_path = resource_path.split(".").clone();
                let mut suffix = "unknown";
                if let Some(_) = split_resource_path.next() {
                    if let Some(second) = split_resource_path.next() {
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
                        format!("could not read resource content: {}", e).to_string(),
                    )),
                ),
            ),
        }
    } else {
        status::Custom(
            Status::BadRequest,
            (
                ContentType::JSON,
                BytesOrError::Error(make_bad_json_data_response("bad resource path".to_string())),
            ),
        )
    }
}
