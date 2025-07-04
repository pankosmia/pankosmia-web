use crate::utils::json_responses::{make_bad_json_data_response, make_good_json_data_response};
use rocket::http::{ContentType, Status};
use rocket::response::status;

pub(crate) fn string_response(
    status_code: Status,
    mime_type: ContentType,
    content: String,
) -> status::Custom<(ContentType, String)> {
    status::Custom(status_code, (mime_type, content))
}

pub(crate) fn ok_json_response(content: String) -> status::Custom<(ContentType, String)> {
    string_response(Status::Ok, ContentType::JSON, content)
}

pub(crate) fn ok_ok_json_response() -> status::Custom<(ContentType, String)> {
    ok_json_response(make_good_json_data_response("ok".to_string()))
}

pub(crate) fn ok_html_response(content: String) -> status::Custom<(ContentType, String)> {
    string_response(Status::Ok, ContentType::HTML, content)
}

pub(crate) fn not_ok_json_response(
    status_code: Status,
    content: String,
) -> status::Custom<(ContentType, String)> {
    string_response(status_code, ContentType::JSON, content)
}

pub(crate) fn not_ok_bad_repo_json_response() -> status::Custom<(ContentType, String)> {
    not_ok_json_response(
        Status::BadRequest,
        make_bad_json_data_response("bad repo path".to_string()),
    )
}
