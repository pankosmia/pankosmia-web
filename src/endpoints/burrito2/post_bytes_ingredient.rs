use rocket::fs::TempFile;
use rocket::{post, State};
use rocket::form::{Form, FromForm};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use std::path::PathBuf;
use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::response::{not_ok_json_response, ok_ok_json_response};

#[derive(FromForm)]
pub struct Upload<'f> {
    file: TempFile<'f>
}

#[post("/ingredient/bytes/<repo_path..>?<ipath>", format = "multipart/form-data", data = "<form>")]
pub async fn post_bytes_ingredient(state: &State<AppSettings>,
                                   repo_path: PathBuf,
                                   ipath: String,
                                   mut form: Form<Upload<'_>>, ) -> status::Custom<(ContentType, String)> {
    let file_name = "/tmp/".to_string() + &ipath;
    match form.file.persist_to(file_name).await {
        Ok(_) => ok_ok_json_response(),
        Err(e) => not_ok_json_response(
            Status::InternalServerError,
            make_bad_json_data_response(format!("Could not write: {}", e)),
        )
    }
}