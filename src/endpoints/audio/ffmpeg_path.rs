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
/// Renvoie le chemin du binaire ffmpeg téléchargé localement par l'app desktop
/// (`~/pankosmia/_assets/ffmpeg/**`), que le client peut ensuite transmettre
/// aux endpoints de compilation via le champ `ffmpeg_path`.
///
/// `payload.path` vaut `null` si aucun ffmpeg téléchargé n'est trouvé (le
/// client s'appuiera alors sur le ffmpeg système).
#[get("/ffmpeg-path")]
pub fn ffmpeg_path() -> status::Custom<(ContentType, String)> {
    json_payload_response(Status::Ok, json!({ "path": find_bundled_ffmpeg() }))
}
