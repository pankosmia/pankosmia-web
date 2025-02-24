use rocket::{catch, Request};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use crate::utils::json_responses::make_bad_json_data_response;

#[catch(404)]
pub(crate) fn not_found_catcher(req: &Request<'_>) -> status::Custom<(ContentType, String)> {
    status::Custom(
        Status::NotFound,
        (
            ContentType::JSON,
            make_bad_json_data_response(format!("Resource {} was not found", req.uri()))
                .to_string(),
        ),
    )
}

#[catch(default)]
pub(crate) fn default_catcher(req: &Request<'_>) -> status::Custom<(ContentType, String)> {
    status::Custom(
        Status::InternalServerError,
        (
            ContentType::JSON,
            make_bad_json_data_response(format!("unknown error while serving {}", req.uri()))
                .to_string(),
        ),
    )
}
