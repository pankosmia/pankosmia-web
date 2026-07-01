use std::env;
use std::path::PathBuf;
use walkdir::WalkDir;

/// Nom de l'exécutable ffmpeg selon l'OS.
fn ffmpeg_executable_name() -> &'static str {
    match env::consts::OS {
        "windows" => "ffmpeg.exe",
        _ => "ffmpeg",
    }
}

/// Dossier où l'app desktop télécharge ffmpeg : `~/pankosmia/_assets/ffmpeg`.
/// Renvoie `None` si le home dir est introuvable.
fn bundled_ffmpeg_base_dir() -> Option<PathBuf> {
    home::home_dir().map(|h| h.join("pankosmia").join("_assets").join("ffmpeg"))
}

/// Cherche récursivement le binaire ffmpeg téléchargé localement par l'app
/// desktop dans `~/pankosmia/_assets/ffmpeg/**`. Renvoie son chemin absolu s'il
/// existe. La recherche est récursive car selon l'OS/le build le binaire est
/// niché dans un sous-dossier (ex. `.../7.1.1/bin/ffmpeg.exe` sur Windows).
pub(crate) fn find_bundled_ffmpeg() -> Option<String> {
    let base = bundled_ffmpeg_base_dir()?;
    if !base.is_dir() {
        return None;
    }
    let exe = ffmpeg_executable_name();
    WalkDir::new(&base)
        .into_iter()
        .filter_map(|e| e.ok())
        .find(|e| e.file_type().is_file() && e.file_name().to_str() == Some(exe))
        .and_then(|e| e.path().to_str().map(str::to_string))
}
