use std::sync::atomic::Ordering;
use crate::utils::json_responses::{make_bad_json_data_response};
use crate::utils::response::{not_ok_json_response, not_ok_offline_json_response, ok_json_response};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::get;
use ffmpeg_sidecar::download::auto_download;
use ffmpeg_sidecar::version::ffmpeg_version;
use serde_json::json;
use crate::static_vars::NET_IS_ENABLED;

/// *`POST /ffmpeg/download`*
///
/// Typically mounted as **`/audio/ffmpeg/download`**
///
/// Downloads the ffmpeg binary
#[get("/ffmpeg/download")]
pub async fn download_ffmpeg() -> status::Custom<(ContentType, String)> {
    if !NET_IS_ENABLED.load(Ordering::Relaxed) {
        return not_ok_offline_json_response();
    }
    match ffmpeg_version() {
        Ok(version) => {
            let response_json = json!({"is_good": true, "payload": {"new_download": false, "ffmpeg_version": version}});
            ok_json_response(serde_json::to_string(&response_json).unwrap())
        },
        Err(_) => {
            match auto_download(){
                Ok(_) => {
                    let ffmpeg_version = ffmpeg_version().unwrap();
                    let response_json = json!({"is_good": true, "payload": {"new_download": true, "ffmpeg_version": ffmpeg_version}});
                    ok_json_response(serde_json::to_string(&response_json).unwrap())
                },
                Err(e) => {
                    not_ok_json_response(
                        Status::InternalServerError,
                        make_bad_json_data_response(format!("ffmpeg_sidecar download failed: {}", e))
                    )
                }
            }
        }
    }
}