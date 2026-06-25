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

/// *`POST /compile-chapter/<repo_path>`*
///
/// Typically mounted as **`/audio/compile-chapter/<repo_path>`**
///
/// Assemble un chapitre complet à partir des mp3 de paragraphes déjà rendus par
/// `compile_audio`. À appeler **après** avoir compilé chaque paragraphe.
///
/// Pour le chapitre `CC`, on scanne :
///   - `audio_content/[{book}/]{CC}-{PP}/{CC}-{PP}.mp3`
///
/// On concatène tous les paragraphes trouvés, triés par numéro de paragraphe,
/// en un seul mp3 :
///   - `audio_content/[{book}/]{CC}.mp3`
///
/// Les mp3 de paragraphes étant déjà aux normes door43 (mêmes sample rate /
/// canaux / bitrate), on utilise le démuxeur `concat` de ffmpeg en `-c copy` :
/// pas de ré-encodage, donc pas de perte de qualité.
///
/// Example body:
///
/// `{"chapter": 1}`  ou  `{"chapter": 1, "book": "tit"}`
#[derive(Deserialize)]
pub struct CompileChapterAudioForm {
    chapter: u32,
    book: Option<String>,
    ffmpeg_path: Option<String>,
}

#[post("/compile-chapter/<repo_path..>", format = "json", data = "<json_form>")]
pub fn compile_chapter_audio(
    state: &State<AppSettings>,
    repo_path: PathBuf,
    json_form: Json<CompileChapterAudioForm>,
) -> status::Custom<(ContentType, String)> {

    let ffmpeg_path = json_form
        .ffmpeg_path
        .as_deref()
        .map(str::trim)
        .unwrap_or("");
    let mut cmd = if ffmpeg_version().is_ok() {
        FfmpegCommand::new()
    } else if !ffmpeg_path.is_empty() {
        FfmpegCommand::new_with_path(ffmpeg_path)
    } else {
        return not_ok_json_response(
            Status::InternalServerError,
            make_bad_json_data_response("ffmpeg not found".to_string()),
        );
    };

    let path_components: Components<'_> = repo_path.components();
    if !check_path_components(&mut path_components.clone()) {
        return not_ok_bad_repo_json_response();
    }

    let repo_dir = state.repo_dir.lock().unwrap().clone();
    let ingredients_dir = format!(
        "{}{}{}/ingredients",
        repo_dir,
        os_slash_str(),
        repo_path.display()
    );
    if !Path::new(&ingredients_dir).exists() {
        return not_ok_json_response(
            Status::NotFound,
            make_bad_json_data_response(format!("Repo not found: {}", ingredients_dir)),
        );
    }

    let cc = format!("{:02}", json_form.chapter);
    let audio_content_dir = format!("{}/audio_content", ingredients_dir);

    let scan_dir = match &json_form.book {
        Some(b) if !b.is_empty() => format!("{}/{}", audio_content_dir, b),
        _ => audio_content_dir.clone(),
    };

    let entries = match std::fs::read_dir(&scan_dir) {
        Ok(e) => e,
        Err(e) => {
            return not_ok_json_response(
                Status::NotFound,
                make_bad_json_data_response(format!(
                    "Audio content not found: {} ({})",
                    scan_dir, e
                )),
            )
        }
    };

    let prefix = format!("{}-", cc);
    let mut paragraphs: Vec<(u32, String)> = Vec::new();
    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let pp = match name.strip_prefix(&prefix).and_then(|s| s.parse::<u32>().ok()) {
            Some(pp) => pp,
            None => continue,
        };
        let mp3 = format!("{}/{}/{}.mp3", scan_dir, name, name);
        if Path::new(&mp3).exists() {
            paragraphs.push((pp, mp3));
        }
    }

    if paragraphs.is_empty() {
        return not_ok_json_response(
            Status::BadRequest,
            make_bad_json_data_response(format!(
                "Nothing to compile: no paragraph mp3 found for chapter {} in {}",
                cc, scan_dir
            )),
        );
    }
    paragraphs.sort_by_key(|(pp, _)| *pp);

    let list_body: String = paragraphs
        .iter()
        .map(|(_, p)| format!("file '{}'\n", p.replace('\'', "'\\''")))
        .collect();
    let list_path = std::env::temp_dir().join(format!(
        "pankosmia_chapter_{}_{}.txt",
        cc,
        std::process::id()
    ));
    if let Err(e) = std::fs::write(&list_path, list_body) {
        return not_ok_json_response(
            Status::InternalServerError,
            make_bad_json_data_response(format!("Could not write concat list: {}", e)),
        );
    }
    let list_path_str = list_path.to_string_lossy().to_string();

    let book_name = match &json_form.book {
        Some(b) if !b.is_empty() => b.clone(),
        _ => "".to_string(),
    };
    let cc_num: u32 = cc.trim().parse().expect("Chapter number must be a number");

    let output_name = if book_name.trim().is_empty() {
        format!("{:02}", cc_num)
    } else {
        format!("{}_{:03}", book_name.trim(), cc_num)
    };
    let output_path = format!("{}/{}.mp3", scan_dir, output_name);

    cmd.overwrite()
        .args(&["-f", "concat", "-safe", "0"])
        .input(list_path_str.as_str())
        .args(&["-c", "copy"])
        .output(&output_path);

    let run = (|| -> Result<(), String> {
        let mut child = cmd
            .spawn()
            .map_err(|e| format!("ffmpeg spawn error: {}", e))?;
        let iter = child
            .iter()
            .map_err(|e| format!("ffmpeg iter error: {}", e))?;
        for _ in iter {}
        Ok(())
    })();

    let _ = std::fs::remove_file(&list_path);

    if let Err(e) = run {
        return not_ok_json_response(
            Status::InternalServerError,
            make_bad_json_data_response(e),
        );
    }

    if Path::new(&output_path).exists() {
        ok_ok_json_response()
    } else {
        not_ok_json_response(
            Status::InternalServerError,
            make_bad_json_data_response(format!("Failed to create mp3: {}", output_path)),
        )
    }
}
