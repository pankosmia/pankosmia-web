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
use std::collections::BTreeMap;
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
            None => 50
        };
        // println!("Story {}, para(s) {}-{}", json_form.story_n, first_para, last_para);

        // Récupérer toutes les vidéos et filtrer
        let video_content_path = format!("{}/ingredients/video_content", repo_path);
        // Créer un objet du type: {para_n: path}
        let mut video_map = BTreeMap::new();
        for entry in std::fs::read_dir(&video_content_path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() && path.extension().unwrap() == "mp4" {
                // Fichier du type obs-para-01-01.mp4
                let file_name = path.file_name().unwrap().to_str().unwrap();

                let story_n = match file_name.split("-").nth(2).unwrap().parse::<i32>() {
                    Ok(n) => n,
                    Err(_) => continue,
                };
                let para_n = match file_name.split("-").nth(3).unwrap().replace(".mp4", "").parse::<i32>() {
                    Ok(n) => n,
                    Err(_) => continue,
                };

                // Si le para_n est hors de la plage, on continue
                if para_n < first_para || para_n > last_para {
                    continue;
                }
                if story_n != json_form.story_n {
                    continue;
                }
                println!("Story n: {:?}, Para n: {:?}", story_n, para_n);
                video_map.insert(para_n, path);
            }
        }
        
		// Ajouter les inputs en arguments
		let mut args: Vec<String> = Vec::new();
        let mut concat_args = String::new();
		for (para_n, path) in video_map.iter() {
			args.push("-i".to_string());
			args.push(path.display().to_string());
            concat_args.push_str(format!("[{}:v][{}:a]", para_n-1, para_n-1).as_str());
		}
		args.push("-filter_complex".to_string());
		args.push(format!("{}concat=n={}:v=1:a=1[v][a]", concat_args, video_map.len()));
        args.push("-map".to_string());
        args.push("[v]".to_string());
        args.push("-map".to_string());
        args.push("[a]".to_string());
		let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
		println!("Args: {:?}", args);

        // // Créer la vidéo finale
        let video_path = format!("{}/obs-story-{}.mp4", video_content_path, json_form.story_n);
        let video_writer = FfmpegCommand::new()
            .overwrite()
			.args(args_refs)
            .codec_video("libx264")
            .codec_audio("aac")
            .output(video_path)
            .spawn().unwrap()
            .iter().unwrap();

        video_writer.for_each(|_| {});
        
        ok_ok_json_response()
    } else {
        not_ok_bad_repo_json_response()
    }
}
