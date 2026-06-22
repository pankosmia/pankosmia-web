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
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::path::{Components, PathBuf};

const OUTPUT_SAMPLE_RATE: u32 = 44100;
const OUTPUT_CHANNELS: u32 = 1;
const OUTPUT_BITRATE: &str = "64k";

/// *`POST /compile/<repo_path>`*
///
/// Typically mounted as **`/audio/compile/<repo_path>`**
///
/// Rend (« compile ») le projet audio non-destructif (multi-pistes + EDL stocké
/// dans `_project.json`) en un seul mp3 aux normes door43.
///
/// Pour chaque paragraphe, l'éditeur stocke :
///   - un `.webm` par buffer de piste : `audio_content/[{book}/]{CC}-{PP}/{CC}-{PP}_{trackId}.webm`
///   - l'EDL : `audio_content/[{book}/]{CC}-{PP}/{CC}-{PP}_project.json`
///
/// On lit l'EDL, on rejoue chaque segment (atrim sur la source + adelay à sa
/// position timeline), on mixe le tout (amix) et on encode en mp3.
///
/// Example body:
///
/// `{"chapter": 1, "paragraph": 1}`  ou  `{"chapter": 1, "paragraph": 1, "book": "tit"}`
#[derive(Deserialize)]
pub struct CompileAudioForm {
    chapter: u32,
    paragraph: u32,
    book: Option<String>,
}

#[post("/compile/<repo_path..>", format = "json", data = "<json_form>")]
pub fn compile_audio(
    state: &State<AppSettings>,
    repo_path: PathBuf,
    json_form: Json<CompileAudioForm>,
) -> status::Custom<(ContentType, String)> {
    if ffmpeg_version().is_err() {
        return not_ok_json_response(
            Status::InternalServerError,
            make_bad_json_data_response("ffmpeg not found".to_string()),
        );
    }

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
    let pp = format!("{:02}", json_form.paragraph);
    let rel_dir = match &json_form.book {
        Some(b) if !b.is_empty() => format!("audio_content/{}/{}-{}", b, cc, pp),
        _ => format!("audio_content/{}-{}", cc, pp),
    };
    let dir = format!("{}/{}", ingredients_dir, rel_dir);
    let project_path = format!("{}/{}-{}_project.json", dir, cc, pp);

    let project_str = match std::fs::read_to_string(&project_path) {
        Ok(s) => s,
        Err(e) => {
            return not_ok_json_response(
                Status::NotFound,
                make_bad_json_data_response(format!(
                    "Project not found: {} ({})",
                    project_path, e
                )),
            )
        }
    };
    let project: Value = match serde_json::from_str(&project_str) {
        Ok(v) => v,
        Err(e) => {
            return not_ok_json_response(
                Status::BadRequest,
                make_bad_json_data_response(format!("Invalid project json: {}", e)),
            )
        }
    };

    let tracks = match project.get("tracks").and_then(|t| t.as_array()) {
        Some(t) => t,
        None => {
            return not_ok_json_response(
                Status::BadRequest,
                make_bad_json_data_response("Project has no tracks".to_string()),
            )
        }
    };

    let mut input_paths: Vec<String> = Vec::new();
    let mut input_index: HashMap<String, usize> = HashMap::new();

    let mut filters: Vec<String> = Vec::new();
    let mut seg_labels: Vec<String> = Vec::new();

    let webm_path = |id: &str| format!("{}/{}-{}_{}.webm", dir, cc, pp, id);

    let track = match tracks.first() {
        Some(t) => t,
        None => {
            return not_ok_json_response(
                Status::BadRequest,
                make_bad_json_data_response("Project has no tracks".to_string()),
            )
        }
    };
    let track_id = match track.get("id").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => {
            return not_ok_json_response(
                Status::BadRequest,
                make_bad_json_data_response("Main track has no id".to_string()),
            )
        }
    };
    let edl = match track.get("edl").and_then(|v| v.as_array()) {
        Some(e) => e,
        None => {
            return not_ok_json_response(
                Status::BadRequest,
                make_bad_json_data_response("Main track has no edl".to_string()),
            )
        }
    };
    for seg in edl {
        let src_id = seg
            .get("bufferTrackId")
            .and_then(|v| v.as_str())
            .unwrap_or(track_id);
        let src_start = seg.get("srcStart").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let src_end = seg.get("srcEnd").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let v_start = seg.get("vStart").and_then(|v| v.as_f64()).unwrap_or(0.0);
        if src_end <= src_start {
            continue;
        }

        let idx = match input_index.get(src_id) {
            Some(i) => *i,
            None => {
                let p = webm_path(src_id);
                if !Path::new(&p).exists() {
                    continue;
                }
                let i = input_paths.len();
                input_paths.push(p);
                input_index.insert(src_id.to_string(), i);
                i
            }
        };

        let label = format!("s{}", seg_labels.len());
        let delay_ms = (v_start * 1000.0).round() as i64;
        filters.push(format!(
            "[{}:a]atrim=start={}:end={},asetpts=PTS-STARTPTS,adelay={}:all=1[{}]",
            idx, src_start, src_end, delay_ms, label
        ));
        seg_labels.push(label);
    }
    

    if seg_labels.is_empty() {
        return not_ok_json_response(
            Status::BadRequest,
            make_bad_json_data_response("Nothing to compile (no audio segments)".to_string()),
        );
    }

    let (filter_complex, map_label) = if seg_labels.len() == 1 {
        (filters.join(";"), seg_labels[0].clone())
    } else {
        let mix = format!(
            "{}amix=inputs={}:normalize=0[mix]",
            seg_labels
                .iter()
                .map(|l| format!("[{}]", l))
                .collect::<String>(),
            seg_labels.len()
        );
        (format!("{};{}", filters.join(";"), mix), "mix".to_string())
    };
    let map_arg = format!("[{}]", map_label);
    let sample_rate = OUTPUT_SAMPLE_RATE.to_string();
    let channels = OUTPUT_CHANNELS.to_string();

    let output_path = format!("{}/{}-{}.mp3", dir, cc, pp);

    let mut cmd = FfmpegCommand::new();
    cmd.overwrite();
    for p in &input_paths {
        cmd.input(p);
    }
    cmd.args(&["-filter_complex", filter_complex.as_str()])
        .args(&["-map", map_arg.as_str()])
        .args(&["-ar", sample_rate.as_str()])
        .args(&["-ac", channels.as_str()])
        .args(&["-c:a", "libmp3lame", "-b:a", OUTPUT_BITRATE])
        .output(&output_path);

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!("ffmpeg spawn error: {}", e)),
            )
        }
    };
    let iter = match child.iter() {
        Ok(it) => it,
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!("ffmpeg iter error: {}", e)),
            )
        }
    };

    for _ in iter {}

    if Path::new(&output_path).exists() {
        ok_ok_json_response()
    } else {
        not_ok_json_response(
            Status::InternalServerError,
            make_bad_json_data_response(format!("Failed to create mp3: {}", output_path)),
        )
    }
}
