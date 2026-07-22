use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::check_path_components1;
use crate::utils::response::not_ok_json_response;
use rocket::get;
use rocket::http::{ContentType, Status};
use rocket::response::status;
use std::path::{Components, PathBuf};

/// *`GET /framed/<top_margin>/<background_color>/<repo_path>`*
///
/// Typically mounted as **`/html/framed/<top_margin>/<background_color>/<repo_path>`**
///
/// Returns html with a padded iframe containing the specified same-origin path, with the specified top margin (int) and background color (6 hex).
#[get("/framed/<top_margin>/<background_color>/<repo_path..>")]
pub async fn framed_client(
    top_margin: u8,
    background_color: &str,
    repo_path: PathBuf,
) -> status::Custom<(ContentType, String)> {
    if background_color.len() != 6 {
        return not_ok_json_response(
            Status::BadRequest,
            make_bad_json_data_response(format!("BG is not 6 hex values: {}", background_color)),
        );
    };
    let path_components: Components<'_> = repo_path.components();
    if check_path_components1(&mut path_components.clone(), 2) {
        let iframe_html = format!("<iframe
        id=\"Pankosmia framed\"
        scrolling=\"no\"
        style=\"width:100%; height: 100%; border:none; margin:0; padding:0; background-color: #FFFFFF\"
        src=\"/{}\">
        </iframe>",
        repo_path.display());
        let framed_html = format!(
            "<html>
        <head><title>iFrame</title></head>
        <body style=\"padding: 0; margin: 0; padding-top: {}px; background-color: #{}\">{}</body>
        </html>",
            &top_margin, background_color, iframe_html
        );
        status::Custom(Status::Ok, (ContentType::HTML, framed_html))
    } else {
        not_ok_json_response(
            Status::BadRequest,
            make_bad_json_data_response(format!("Bad path: {:?}", repo_path)),
        )
    }
}
