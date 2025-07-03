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

pub(crate) fn ok_html_response(content: String) -> status::Custom<(ContentType, String)> {
    string_response(Status::Ok, ContentType::HTML, content)
}

pub(crate) fn not_ok_json_response(
    status_code: Status,
    content: String,
) -> status::Custom<(ContentType, String)> {
    string_response(status_code, ContentType::JSON, content)
}
