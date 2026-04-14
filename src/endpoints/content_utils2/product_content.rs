use crate::structs::AppSettings;
use crate::utils::paths::os_slash_str;
use crate::utils::response::{ok_json_response};
use rocket::http::{ContentType};
use rocket::response::status;
use rocket::{get, State};

/// *`GET /product/<client_name>`*
///
/// Typically mounted as **`/content-utils/product/<client_name>`**
///
/// Returns a JSON catalog of product-specific resources
/// The catalog is stored in a directory that is populated at run or build time
/// Note that there should only be one set of product content at any one time.
/// It is not an error for no product content or product content index to exist for a given client.
#[get("/product/<client_name>")]
pub async fn product_content_catalog(
    state: &State<AppSettings>,
    client_name: String,
) -> status::Custom<(ContentType, String)> {
    let path_to_serve = format!(
        "{}{}{}{}{}{}{}{}{}",
        &state.app_resources_dir,
        os_slash_str(),
        "app_resources",
        os_slash_str(),
        "product",
        os_slash_str(),
        client_name,
        os_slash_str(),
        "index.json"
    );

    match std::fs::read_to_string(path_to_serve) {
        Ok(v) => ok_json_response(v),
        Err(_) => ok_json_response("[]".to_string()),
    }
}
