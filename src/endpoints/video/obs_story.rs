use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::{check_path_components, os_slash_str};
use crate::utils::response::{
    not_ok_bad_repo_json_response, not_ok_json_response, ok_ok_json_response,
};
use ffmpeg_sidecar::command::FfmpegCommand;
use ffmpeg_sidecar::version::ffmpeg_version;
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{post, State};
use serde::Deserialize;
use std::path::Path;
use std::path::{Components, PathBuf};

/// *`POST /obs-story/<repo_path>`*
///
/// Typically mounted as **`/video/obs-story/<repo_path>`**
///
/// Example body:
///
/// `{"story_n": 1, "from_para_n": 3, "to_para_n": 7}`

#[derive(Deserialize)]
pub struct ObsStoryForm {
    story_n: i32,
    from_para_n: Option<i32>,
    to_para_n: Option<i32>,
}

#[post("/obs-story/<repo_path..>", format = "json", data = "<json_form>")]
pub fn obs_story_video(
    state: &State<AppSettings>,
    repo_path: PathBuf,
    json_form: Json<ObsStoryForm>,
) -> status::Custom<(ContentType, String)> {
    match ffmpeg_version() {
        Ok(_) => {}
        Err(_) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response("ffmpeg not found".to_string()),
            )
        }
    }
    let path_components: Components<'_> = repo_path.components();
    if check_path_components(&mut path_components.clone()) {
        let repo_dir = state.repo_dir.lock().unwrap().clone();
        let repo_path = format!(
            "{}{}{}",
            &repo_dir,
            os_slash_str(),
            &repo_path.display().to_string()
        );

        // Verifier si le chemin est valide
        if !Path::new(&repo_path).exists() {
            return not_ok_json_response(
                Status::NotFound,
                make_bad_json_data_response(format!("Repo not found: {}", repo_path)),
            );
        }
        let first_para = match json_form.from_para_n {
            Some(n) => n,
            None => 0
        };
        let last_para = match json_form.to_para_n {
            Some(n) => n,
            None => 999
        };
        println!("Story {}, para(s) {}-{}", json_form.story_n, first_para, last_para);
        ok_ok_json_response()
    } else {
        not_ok_bad_repo_json_response()
    }
}
