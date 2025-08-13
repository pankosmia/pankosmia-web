use std::sync::atomic::Ordering;
use crate::utils::json_responses::{make_bad_json_data_response};
use crate::utils::response::{not_ok_json_response, not_ok_offline_json_response, ok_ok_json_response};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::get;
use ffmpeg_sidecar::version::ffmpeg_version;
use ffmpeg_sidecar::command::FfmpegCommand;
use crate::static_vars::NET_IS_ENABLED;

/// *`POST /ffmpeg/test`*
///
/// Typically mounted as **`/audio/ffmpeg/test`**
///
/// Tests ffmpeg
#[get("/ffmpeg/test")]
pub async fn test_ffmpeg() -> status::Custom<(ContentType, String)> {
    if !NET_IS_ENABLED.load(Ordering::Relaxed) {
        return not_ok_offline_json_response();
    }
    match ffmpeg_version() {
        Ok(_) => {
            let iter = FfmpegCommand::new() // <- Builder API like `std::process::Command`
                .testsrc()  // <- Discoverable aliases for FFmpeg args
                .rawvideo() // <- Convenient argument presets
                .spawn().unwrap()   // <- Ordinary `std::process::Child`
                .iter().unwrap();   // <- Blocking iterator over logs and output

            // Use a regular "for" loop to read decoded video data
            for frame in iter.filter_frames() {
                println!("frame: {}x{}", frame.width, frame.height);
                let _pixels: Vec<u8> = frame.data; // <- raw RGB pixels!
            }
            ok_ok_json_response()
        },
        Err(_) => {

                    not_ok_json_response(
                        Status::InternalServerError,
                        make_bad_json_data_response("ffmpeg_sidecar not found".to_string())
                    )
                }
            }
        }