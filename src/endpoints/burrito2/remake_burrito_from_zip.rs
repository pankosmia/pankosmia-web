use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::{check_path_components, os_slash_str};
use crate::utils::response::{
    not_ok_bad_repo_json_response, not_ok_json_response, ok_ok_json_response,
};
use crate::utils::zip::unpack_zip_file;
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{post, State};
use std::fs;
use std::fs::copy;
use std::path::{Components, Path, PathBuf};
use tempfile::Builder;
use tempfile::NamedTempFile;

/// Remove everything in a folder (does not delete the folder)
fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

/// Remove only `.git` folder and any git-related files inside the folder
fn remove_git_files(path: &Path) -> Result<(), std::io::Error> {
    if path.exists() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();

            // Remove the .git folder entirely
            if entry_path.is_dir() && entry_path.file_name() == Some(".git".as_ref()) {
                fs::remove_dir_all(&entry_path)?;
            }

            // Optionally, remove other git-related files (like .gitignore, .gitattributes)
            if entry_path.is_file() {
                if let Some(name) = entry_path.file_name() {
                    if name == ".gitignore" || name == ".gitattributes" {
                        fs::remove_file(&entry_path)?;
                    }
                }
            }

            // If it’s a subdirectory, recurse
            if entry_path.is_dir() {
                remove_git_files(&entry_path)?;
            }
        }
    }
    Ok(())
}

/// Remove everything in a folder, but keep all git related files
fn clear_directory_keep_git(path: &Path) -> Result<(), std::io::Error> {
    if path.exists() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();

            // Skip .git folders
            if entry_path.is_dir() && entry_path.file_name() == Some(".git".as_ref()) {
                continue;
            }

            if entry_path.is_dir() {
                fs::remove_dir_all(&entry_path)?;
            } else {
                fs::remove_file(&entry_path)?;
            }
        }
    }
    Ok(())
}
/// Returns true if the zip looks a bit like a burrito
fn check_burrito_dir(path: &Path) -> bool {
    let mut metadata_found = false;
    let mut ingredients_found = false;

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();

            if entry_path.is_file() && entry_path.file_name() == Some("metadata.json".as_ref()) {
                metadata_found = true;
            }

            if entry_path.is_dir() && entry_path.file_name() == Some("ingredients".as_ref()) {
                ingredients_found = true;
            }

            // Early exit if both found
            if metadata_found && ingredients_found {
                return true;
            }
        }
    }

    metadata_found && ingredients_found
}
/// *`POST /remake_burrito_from_zip/<repo_path>`*
///
/// Typically mounted as **`/burrito/remake_burrito_from_zip/<repo_path>`**
///
/// If the zip looks like a burrito replace the
///     current content of the repo with the content of the zip
#[post("/remake_burrito_from_zip/<temp_id>/<repo_path..>")]
pub async fn remake_burrito_from_zip(
    state: &State<AppSettings>,
    temp_id: &str,
    repo_path: PathBuf,
) -> status::Custom<(ContentType, String)> {
    let path_components: Components<'_> = repo_path.components();
    let full_repo_path = format!(
        "{}{}{}",
        state.repo_dir.lock().unwrap(),
        os_slash_str(),
        &repo_path.display().to_string()
    );
    if check_path_components(&mut path_components.clone()) {
        if !Path::new(&full_repo_path).exists() {
            return not_ok_json_response(
                Status::BadRequest,
                make_bad_json_data_response(format!("Repo does not already exist")),
            );
        }
        let temp_zip_path = format!(
            "{}{}temp{}{}",
            state.working_dir.clone(),
            os_slash_str(),
            os_slash_str(),
            &temp_id
        );
        let temp_path = Path::new(&temp_zip_path);
        if !temp_path.exists() {
            return not_ok_json_response(
                Status::BadRequest,
                make_bad_json_data_response(format!("Temp zip with UUID {} not found", temp_id)),
            );
        }
        let temp_dir_base = format!("{}{}temp", state.working_dir.clone(), os_slash_str(),);
        // Copy upload to temp file we manage
        let named_temp = NamedTempFile::new().expect("tempfile for zip check");
        copy(temp_path, named_temp.path()).expect("copy temp zip to NamedTempFile");
        let unpack_dir = Builder::new()
            .prefix("repo_unpack_")
            .tempdir_in(&temp_dir_base)
            .expect("create temp unpack dir");

        let unpack_path = unpack_dir.path().to_path_buf();

        // 2. Unpack zip INTO TEMP DIR (NOT repo)
        match unpack_zip_file(named_temp, unpack_path.to_string_lossy().to_string(), Some(1)).await {
            Ok(_) => (),
            Err(e) => {
                return not_ok_json_response(
                    Status::InternalServerError,
                    make_bad_json_data_response(format!("Unpack failed: {}", e)),
                );
            }
        }

        // 3. Check burrito in temp dir
        if !check_burrito_dir(&unpack_path) {
            return not_ok_json_response(
                Status::BadRequest,
                make_bad_json_data_response("Zip is not a burrito".to_string()),
            );
        }
        // 4. Remove git files from temp (important BEFORE copy)
        if let Err(e) = remove_git_files(&unpack_path) {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!("Failed to clean git files: {}", e)),
            );
        }

        // 5. Clear repo safely
        let repo_path = Path::new(&full_repo_path);
        if let Err(e) = clear_directory_keep_git(repo_path) {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!("Could not clear repo: {}", e)),
            );
        }

        // 6. Copy temp → repo
        if let Err(e) = copy_dir_all(&unpack_path, repo_path) {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!("Failed to copy new content: {}", e)),
            );
        }
        ok_ok_json_response()
        // Unpack zip

        // Check burrito
    } else {
        not_ok_bad_repo_json_response()
    }
}
