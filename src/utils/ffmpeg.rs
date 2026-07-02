use std::env;
use std::path::PathBuf;
use walkdir::WalkDir;

fn ffmpeg_executable_name() -> &'static str {
    match env::consts::OS {
        "windows" => "ffmpeg.exe",
        _ => "ffmpeg",
    }
}

/// Directory where the desktop app downloads ffmpeg: `~/<working_dir>/_assets/ffmpeg`.
/// Returns `None` if the home directory cannot be found.
fn bundled_ffmpeg_base_dir(working_dir: String) -> Option<PathBuf> {
    let working_path_buf = PathBuf::from(working_dir);
    Some(working_path_buf.join("pankosmia").join("_assets").join("ffmpeg"))
}

/// Recursively searches `~/pankosmia/_assets/ffmpeg/**` for the ffmpeg binary
/// downloaded locally by the desktop app and returns its absolute path if
/// present. The search is recursive because, depending on the OS/build, the
/// binary is nested in a subdirectory (e.g. `.../7.1.1/bin/ffmpeg.exe` on
/// Windows).
pub(crate) fn find_bundled_ffmpeg(working_dir: String) -> Option<String> {
    let base = bundled_ffmpeg_base_dir(working_dir)?;
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
