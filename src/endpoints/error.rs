use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::response::not_ok_json_response;
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{catch, Request};

#[catch(404)]
pub(crate) fn not_found_catcher(req: &Request<'_>) -> status::Custom<(ContentType, String)> {
    not_ok_json_response(
        Status::NotFound,
        make_bad_json_data_response(format!("Resource {} was not found", req.uri())).to_string(),
    )
}

#[catch(default)]
pub(crate) fn default_catcher(req: &Request<'_>) -> status::Custom<(ContentType, String)> {
    not_ok_json_response(
        Status::InternalServerError,
        make_bad_json_data_response(format!("unknown error while serving {}", req.uri()))
            .to_string(),
    )
}
