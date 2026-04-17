use std::path::Path;

use crate::structs::AppSettings;
use crate::utils::paths::os_slash_str;
use crate::utils::response::ok_json_response;
use rocket::http::ContentType;
use rocket::response::status;
use rocket::{get, State};
use serde_json::json;
use walkdir::WalkDir;

/// *`GET /version`*
///
/// Typically mounted as **`/version`**
///
/// Returns an object containing version information
///
/// `{"pkg_version":"1.2.3"}`
#[get("/version")]
pub fn get_version(state: &State<AppSettings>) -> status::Custom<(ContentType, String)> {
    let crate_version = env!("CARGO_PKG_VERSION");
    let product = &state.product;
    let mut product_resources: Vec<String> = Vec::new();
    // Does product resources dir exist?
    let path_to_walk = format!(
        "{}{}{}{}{}{}{}",
        &state.app_resources_dir,
        os_slash_str(),
        "app_resources",
        os_slash_str(),
        "product",
        os_slash_str(),
        "product_resources"
    );
    if Path::new(&path_to_walk).exists() {
        // If so, walk the directory and collect file paths
        for entry in WalkDir::new(&path_to_walk) {
            let entry_string = entry.unwrap().path().display().to_string();
            if Path::new(&entry_string).is_file() {
                let truncate_prefix = format!("{}/", path_to_walk.clone().replace("\\", "/"));
                let truncated_entry_string = entry_string.replace("\\", "/").replace(&truncate_prefix, "");
                product_resources.push(truncated_entry_string.clone());
            }
        }
    }
    let json_value = json!({
        "pkg_version": crate_version,
        "product_name": product.name,
        "product_short_name": product.short_name,
        "product_version": product.version,
        "product_date_time": product.date_time,
        "product_resources": product_resources
    })
    .to_string();
    ok_json_response(json_value)
}
