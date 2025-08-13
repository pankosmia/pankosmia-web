use std::path::{Components, PathBuf};
use crate::structs::{AppSettings};
use crate::utils::response::{not_ok_bad_repo_json_response, ok_ok_json_response};
use rocket::http::{ContentType};
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{post, State};
use serde::Deserialize;
use crate::utils::paths::{check_path_components, os_slash_str};

/// *`POST /ffmpeg/video/obs-para/<repo_path>`*
///
/// Typically mounted as **`/audio/ffmpeg/video/obs-para/<repo_path>`**

#[derive(Deserialize)]
pub struct ObsParaForm {
    story_n: i32,
    para_n: i32,
    audio_path: String
}

#[post("/ffmpeg/video/obs-para/<repo_path..>", format = "json", data = "<json_form>")]
pub fn obs_para_video(
    state: &State<AppSettings>,
    repo_path: PathBuf,
    json_form: Json<ObsParaForm>,
) -> status::Custom<(ContentType, String)> {
    let path_components: Components<'_> = repo_path.components();
    if check_path_components(&mut path_components.clone()) {
        let repo_path = format!(
            "{}{}{}",
            state.repo_dir.lock().unwrap().clone(),
            os_slash_str(),
            &repo_path.display().to_string()
        );
        println!("Repo {} Story {}, Para {}, audio path {}", repo_path, json_form.story_n, json_form.para_n, json_form.audio_path);
        ok_ok_json_response()
    } else {
        not_ok_bad_repo_json_response()
    }
}
