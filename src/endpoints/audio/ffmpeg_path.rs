use crate::utils::ffmpeg::find_bundled_ffmpeg;
use crate::utils::response::json_payload_response;
use rocket::get;
use rocket::http::{ContentType, Status};
use rocket::response::status;
use serde_json::json;

/// *`GET /ffmpeg-path`*
///
/// Typically mounted as **`/audio/ffmpeg-path`**
///
/// Returns the path to the ffmpeg binary downloaded locally by the desktop app
/// (`~/pankosmia/_assets/ffmpeg/**`), which the client can then forward to the
/// compilation endpoints via the `ffmpeg_path` field.
///
/// `payload.path` is `null` when no downloaded ffmpeg is found (the client then
/// falls back to the system ffmpeg).
#[get("/ffmpeg-path")]
pub fn ffmpeg_path() -> status::Custom<(ContentType, String)> {
    json_payload_response(Status::Ok, json!({ "path": find_bundled_ffmpeg() }))
}
