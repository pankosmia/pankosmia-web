use std::path::{Components, PathBuf};
use crate::structs::{AppSettings};
use crate::utils::response::{not_ok_json_response, not_ok_bad_repo_json_response, ok_ok_json_response};
use crate::utils::json_responses::{make_bad_json_data_response};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{post, State};
use serde::Deserialize;
use crate::utils::paths::{check_path_components, os_slash_str};
use ffmpeg_sidecar::version::ffmpeg_version;
use ffmpeg_sidecar::command::FfmpegCommand;
use std::path::Path;

/// *`POST /obs-para/<repo_path>`*
///
/// Typically mounted as **`/video/obs-para/<repo_path>`**
///
/// Example body:
///
/// `{"story_n": 1, "para_n": 1, "audio_path": "audio_content/01-01/01-01_0_1.mp3"}`

#[derive(Deserialize)]
pub struct ObsParaForm {
    story_n: i32,
    para_n: i32,
    audio_path: String
}

#[post("/obs-para/<repo_path..>", format = "json", data = "<json_form>")]
pub fn obs_para_video(
    state: &State<AppSettings>,
    repo_path: PathBuf,
    json_form: Json<ObsParaForm>,
) -> status::Custom<(ContentType, String)> {
    match ffmpeg_version() {
        Ok(_) => {},
        Err(_) => return not_ok_json_response(
            Status::InternalServerError,
            make_bad_json_data_response("ffmpeg not found".to_string()),
        ),
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
                make_bad_json_data_response(format!("Repo not found: {}", &repo_path))
            );
        }

        // Formater story et para pour que les chiffres inferieurs a 10 aient un 0 devant
        let story_string = if json_form.story_n < 10 {
            format!("0{}", json_form.story_n)
        } else {
            json_form.story_n.to_string()
        };
        let para_string = if json_form.para_n < 10 {
            format!("0{}", json_form.para_n)
        } else {
            json_form.para_n.to_string()
        };

        // Chemin du fichier video
        let audio_path = format!("{}/ingredients/{}", repo_path, json_form.audio_path);
        // Vérifier si le fichier video existe
        if !Path::new(&audio_path).exists() {
            return not_ok_json_response(
                Status::NotFound,
                make_bad_json_data_response(format!("Audio file not found: {}", &audio_path))
            );
        }
        
        // Chemin de l'image
        let images_path = format!("{}/git.door43.org/uW/obs_images_360/ingredients/360px/obs-en-{}-{}.jpg", repo_dir, story_string, para_string);
        // Vérifier si le fichier image existe
        if !Path::new(&images_path).exists() {
            println!("Image file not found: {}", &images_path);
            return not_ok_json_response(
                Status::NotFound,
                make_bad_json_data_response(format!("Image file not found: {}", &images_path))
            );
        }

        // Création de la vidéo
        match ffmpeg_version() {
            Ok(_) => {
                
                let video_content_path = format!("{}/ingredients/video_content", repo_path);
                if !Path::new(&video_content_path).exists() {
                    std::fs::create_dir_all(&video_content_path).unwrap();
                }
                let video_path = format!("{}/obs-en-{}-{}.mp4", video_content_path, story_string, para_string);

                // println!("Video path: {}", video_path);
                // Créer la vidéo et attendre la fin du traitement
                let iter = FfmpegCommand::new()
                    .overwrite()
                    .args(&["-loop", "1"])
                    .args(&["-framerate", "25"])
                    .input(&images_path)
                    .input(&audio_path)
                    .args(&["-shortest"])
                    .codec_video("libx264")
                    .pix_fmt("yuv420p")
                    .args(&["-movflags", "+faststart"])
                    .codec_audio("aac")
                    .output(&video_path)
                    // .print_command()
                    .spawn().unwrap()
                    .iter().unwrap();

                for _ in iter {
                    // On consomme les événements/logs jusqu'à la fin
                }

                // Vérifier que le fichier a bien été créé
                if Path::new(&video_path).exists() {
                    ok_ok_json_response()
                } else {
                    not_ok_json_response(
                        Status::InternalServerError,
                        make_bad_json_data_response(format!("Échec de création de la vidéo: {}", video_path))
                    )
                }
            },
            Err(_) => {
    
                not_ok_json_response(
                    Status::InternalServerError,
                    make_bad_json_data_response("ffmpeg_sidecar not found".to_string())
                )
            }
        }
    } else {
        not_ok_bad_repo_json_response()
    }
}
