// REPO OPERATIONS

use md5;
use std::collections::{BTreeMap};
use std::path::{Components, PathBuf};
use std::sync::atomic::Ordering;
use git2::{Repository, StatusOptions};
use rocket::{get, post, State};
use rocket::serde::json::Json;
use rocket::http::{ContentType, Status};
use rocket::response::status;
use serde_json::{json, Value};
use crate::static_vars::NET_IS_ENABLED;
use crate::structs::{AppSettings, BurritoMetadataIngredient, GitStatusRecord, NewScriptureBookForm};
use crate::structs::BurritoMetadata;
use crate::utils::files::load_json;
use crate::utils::json_responses::{make_bad_json_data_response, make_good_json_data_response};
use crate::utils::paths::{check_path_components, check_local_path_components, os_slash_str};

/// *`GET /list-local-repos`*
///
/// Typically mounted as **`/git/list-local-repos`**
///
/// Returns a JSON array of local repo paths.
///
/// `["git.door43.org/BurritoTruck/fr_psle"]`
#[get("/list-local-repos")]
pub fn list_local_repos(state: &State<AppSettings>) -> status::Custom<(ContentType, String)> {
    let root_path = state.repo_dir.lock().unwrap().clone();
    let server_paths = std::fs::read_dir(root_path).unwrap();
    let mut repos: Vec<String> = Vec::new();
    for server_path in server_paths {
        let uw_server_path_ob = server_path.unwrap().path();
        let uw_server_path_ob2 = uw_server_path_ob.clone();
        let server_leaf = uw_server_path_ob2.file_name().unwrap();
        for org_path in std::fs::read_dir(uw_server_path_ob).unwrap() {
            let uw_org_path_ob = org_path.unwrap().path();
            let uw_org_path_ob2 = uw_org_path_ob.clone();
            let org_leaf = uw_org_path_ob2.file_name().unwrap();
            for repo_path in std::fs::read_dir(uw_org_path_ob).unwrap() {
                let uw_repo_path_ob = repo_path.unwrap().path();
                let repo_leaf = uw_repo_path_ob.file_name().unwrap();
                let repo_url_string = format!(
                    "{}/{}/{}",
                    server_leaf.to_str().unwrap(),
                    org_leaf.to_str().unwrap(),
                    repo_leaf.to_str().unwrap()
                );
                repos.push(repo_url_string);
            }
        }
    }
    let quoted_repos: Vec<String> = repos
        .into_iter()
        .map(|str: String| format!("{}", str))
        .collect();
    status::Custom(
        Status::Ok,
        (
            ContentType::JSON,
            serde_json::to_string(&quoted_repos).unwrap(),
        ),
    )
}

/// *`GET /add-and-commit/<repo_path>`*
///
/// Typically mounted as **`/git/add-and-commit/<repo_path>`**
///
/// Adds and commits modified files for a given repo.
#[get("/add-and-commit/<repo_path..>")]
pub async fn add_and_commit(
    state: &State<AppSettings>,
    repo_path: PathBuf,
) -> status::Custom<(ContentType, String)> {
    let repo_path_string: String = state.repo_dir.lock().unwrap().clone()
        + os_slash_str()
        + &repo_path.display().to_string().clone();
    let result = match Repository::open(repo_path_string) {
        Ok(repo) => {
            repo.index()
                .unwrap()
                .add_all(&["."], git2::IndexAddOption::DEFAULT, None)
                .unwrap();
            repo.index().unwrap().write().unwrap();
            let mut index = repo.index().unwrap();
            let oid = index.write_tree().unwrap();
            let signature = repo.signature().unwrap();
            let parent_commit = repo.head().unwrap().peel_to_commit().unwrap();
            let tree = repo.find_tree(oid).unwrap();
            repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                "Updated by Pithekos",
                &tree,
                &[&parent_commit],
            )
                .unwrap();
            status::Custom(
                Status::Ok,
                (
                    ContentType::JSON,
                    make_good_json_data_response("ok".to_string()),
                ),
            )
        }
        Err(e) => status::Custom(
            Status::InternalServerError,
            (
                ContentType::JSON,
                make_bad_json_data_response(
                    format!("could not add/commit repo: {}", e).to_string(),
                ),
            ),
        ),
    };
    result
}

/// *`GET /fetch-repo/<repo_path>`*
///
/// Typically mounted as **`/git/fetch-repo/<repo_path>`**
///
/// Makes a local clone of a repo from the given repo path.
#[get("/fetch-repo/<repo_path..>")]
pub async fn fetch_repo(
    state: &State<AppSettings>,
    repo_path: PathBuf,
) -> status::Custom<(ContentType, String)> {
    if !NET_IS_ENABLED.load(Ordering::Relaxed) {
        return status::Custom(
            Status::Unauthorized,
            (
                ContentType::JSON,
                make_bad_json_data_response("offline mode".to_string()),
            ),
        );
    }
    let mut path_components: Components<'_> = repo_path.components();
    if check_path_components(&mut path_components.clone()) {
        let source = path_components
            .next()
            .unwrap()
            .as_os_str()
            .to_str()
            .unwrap();
        let org = path_components
            .next()
            .unwrap()
            .as_os_str()
            .to_str()
            .unwrap();
        let mut repo = path_components
            .next()
            .unwrap()
            .as_os_str()
            .to_str()
            .unwrap()
            .to_string();
        if repo.ends_with(".git") {
            let repo_vec = repo.split(".").collect::<Vec<&str>>();
            let short_repo = &repo_vec[0..repo_vec.len() - 1];
            let short_repo_string = short_repo.join("/");
            repo = short_repo_string.as_str().to_owned();
        }
        let url = "https://".to_string() + &repo_path.display().to_string().replace("\\", "/");
        match Repository::clone(
            &url,
            state.repo_dir.lock().unwrap().clone()
                + os_slash_str()
                + source
                + os_slash_str()
                + org
                + os_slash_str()
                + repo.as_str(),
        ) {
            Ok(_repo) => status::Custom(
                Status::Ok,
                (
                    ContentType::JSON,
                    make_good_json_data_response("ok".to_string()),
                ),
            ),
            Err(e) => {
                println!("Error:{}", e);
                status::Custom(
                    Status::BadRequest,
                    (
                        ContentType::JSON,
                        make_bad_json_data_response(
                            format!("could not clone repo: {}", e).to_string(),
                        ),
                    ),
                )
            }
        }
    } else {
        status::Custom(
            Status::BadRequest,
            (
                ContentType::JSON,
                make_bad_json_data_response("bad repo path".to_string()),
            ),
        )
    }
}

/// *`POST /new-scripture-book/<repo_path>`*
///
/// Typically mounted as **`/git/new-scripture-path/<repo_path>`**
///
/// Adds a Scripture book to a local repo at the given repo path.
///
///  It requires the following fields as a JSON body:
/// - book_code (string)
/// - book_title (string)
/// - book_abbr (string)
/// - add_cv (boolean)
#[allow(irrefutable_let_patterns)]
#[post("/new-scripture-book/<repo_path..>",
    format = "json",
    data = "<json_form>")]
pub async fn new_scripture_book(
    state: &State<AppSettings>,
    repo_path: PathBuf,
    json_form: Json<NewScriptureBookForm>,
) -> status::Custom<(ContentType, String)> {
    let path_components: Components<'_> = repo_path.components();
    if check_local_path_components(&mut path_components.clone()) {
        // Read metadata
        let repo_dir_path = state.repo_dir.lock().unwrap().clone();
        let repo_name = path_components.skip(2).next().unwrap().as_os_str().to_str().unwrap();
        let path_to_repo_metadata = format!(
            "{}{}_local_{}_local_{}{}{}metadata.json",
            repo_dir_path,
            os_slash_str(),
            os_slash_str(),
            os_slash_str(),
            &repo_name,
            os_slash_str(),
        );
        let metadata_string = match std::fs::read_to_string(&path_to_repo_metadata) {
            Ok(v) => v,
            Err(e) => return status::Custom(
                Status::InternalServerError,
                (
                    ContentType::JSON,
                    make_bad_json_data_response(format!("Could not load metadata as string: {}", e)),
                ),
            )
        };
        // Make struct from metadata
        let metadata_struct: BurritoMetadata = match serde_json::from_str(&metadata_string) {
            Ok(v) => v,
            Err(e) => {
                return status::Custom(
                    Status::InternalServerError,
                    (
                        ContentType::JSON,
                        make_bad_json_data_response(format!("Could not parse metadata: {}", e)),
                    ),
                );
            }
        };
        // Check new book isn't already there
        let new_ingredients_path = format!("ingredients/{}.usfm", &json_form.book_code);
        if let ingredients = metadata_struct.ingredients.lock().unwrap() {
            if ingredients.contains_key(&new_ingredients_path) {
                return status::Custom(
                    Status::BadRequest,
                    (
                        ContentType::JSON,
                        make_bad_json_data_response(format!("Book '{}' already exists in metadata", &json_form.book_code)),
                    ),
                );
            }
        }
        // Make USFM with optional cv
        let path_to_usfm_template = format!(
            "{}{}templates{}content_templates{}text_translation{}book.usfm",
            &state.app_resources_dir,
            os_slash_str(),
            os_slash_str(),
            os_slash_str(),
            os_slash_str(),
        );
        let mut usfm_string = match std::fs::read_to_string(&path_to_usfm_template) {
            Ok(v) => v,
            Err(e) => return status::Custom(
                Status::InternalServerError,
                (
                    ContentType::JSON,
                    make_bad_json_data_response(format!("Could not load USFM template as string: {}", e)),
                ),
            )
        };
        usfm_string = usfm_string
            .replace("%%BOOKCODE%%", json_form.book_code.clone().as_str())
            .replace("%%BOOKNAME%%", json_form.book_title.clone().as_str())
            .replace("%%CONTENTNAME%%", &repo_name)
            .replace("%%BOOKABBR%%", json_form.book_abbr.clone().as_str());
        // - If ve
        if json_form.add_cv {
            // Load vrs file from repo
            let path_to_repo_vrs = format!(
                "{}{}_local_{}_local_{}{}{}ingredients{}vrs.json",
                repo_dir_path,
                os_slash_str(),
                os_slash_str(),
                os_slash_str(),
                &repo_name,
                os_slash_str(),
                os_slash_str(),
            );
            let versification_ob = match load_json(&path_to_repo_vrs) {
                Ok(j) => j,
                Err(e) => return status::Custom(
                    Status::InternalServerError,
                    (
                        ContentType::JSON,
                        make_bad_json_data_response(format!("Could not load repo versification JSON: {}", e)),
                    ),
                )
            };
            // Generate cv USFM
            let mut cv_bits = Vec::new();
            let max_verses_ob = match &versification_ob["maxVerses"] {
                Value::Object(o) => o,
                _ => return status::Custom(
                    Status::InternalServerError,
                    (
                        ContentType::JSON,
                        make_bad_json_data_response(
                            "Could not find maxVerses in versification JSON for this repo".to_string()
                        )
                    ),
                )
            };
            let book_max_verses_arr = match &max_verses_ob[&json_form.book_code.clone()] {
                Value::Array(a) => a,
                _ => return status::Custom(
                    Status::InternalServerError,
                    (
                        ContentType::JSON,
                        make_bad_json_data_response(
                            format!(
                                "Could not find maxVerses for {} in versification JSON for this repo",
                                json_form.book_code.clone(),
                            )
                        ),
                    ),
                )
            };
            let mut chapter_number = 0;
            for max_verse in book_max_verses_arr {
                chapter_number += 1;
                cv_bits.push(format!("\\c {}", chapter_number));
                cv_bits.push("\\p".to_string());
                let max_verse_number = max_verse.as_str().unwrap().parse::<i32>().unwrap();
                for verse_number in 1..=max_verse_number {
                    cv_bits.push(format!("\\v {}", verse_number));
                }
            }
            // Insert
            usfm_string = usfm_string
                .replace(
                    "%%STUBCONTENT%%",
                    cv_bits.join("\n").as_str(),
                );
        } else {
            usfm_string = usfm_string
                .replace(
                    "%%STUBCONTENT%%",
                    "\\c 1\n\\p\n\\v 1\nFirst verse...",
                );
        }
        // Save USFM
        let path_to_new_book = format!(
            "{}{}_local_{}_local_{}{}{}ingredients{}{}.usfm",
            repo_dir_path,
            os_slash_str(),
            os_slash_str(),
            os_slash_str(),
            &repo_name,
            os_slash_str(),
            os_slash_str(),
            &json_form.book_code
        );
        match std::fs::write(path_to_new_book, &usfm_string) {
            Ok(_) => (),
            Err(e) => return status::Custom(
                Status::InternalServerError,
                (
                    ContentType::JSON,
                    make_bad_json_data_response(format!("Could not write usfm to repo: {}", e)),
                ),
            )
        }
        // Add ingredient record for USFM
        if let mut ingredients = metadata_struct.ingredients.lock().unwrap() {
            let mut new_ingredients = BTreeMap::new();
            for (k, v) in ingredients.iter() {
                new_ingredients.insert(k.clone(), v.clone());
            }
            let ingredient_key = format!("ingredients/{}.usfm", &json_form.book_code);
            let ingredient_struct = BurritoMetadataIngredient {
                checksum: json!({"md5": format!("{:?}", md5::compute(&usfm_string))}),
                mimeType: "text/plain".to_string(),
                scope: Some(json!({json_form.book_code.to_string(): []})),
                size: usfm_string.len(),
            };
            new_ingredients.insert(
                ingredient_key,
                ingredient_struct,
            );
            *ingredients = new_ingredients;
        }
        // Write metadata
        let metadata_output_string = match serde_json::to_string(&metadata_struct) {
            Ok(s) => s,
            Err(e) => {
                return status::Custom(
                    Status::InternalServerError,
                    (
                        ContentType::JSON,
                        make_bad_json_data_response(format!("Could not make metadata as JSON: {}", e)),
                    ),
                )
            }
        };
        match std::fs::write(path_to_repo_metadata, &metadata_output_string) {
            Ok(_) => (),
            Err(e) => return status::Custom(
                Status::InternalServerError,
                (
                    ContentType::JSON,
                    make_bad_json_data_response(format!("Could not write metadata to repo: {}", e)),
                ),
            )
        }
        // Add and commit
        status::Custom(
            Status::Ok,
            (
                ContentType::JSON,
                make_good_json_data_response("ok".to_string()),
            ),
        )
    } else {
        status::Custom(
            Status::Unauthorized,
            (
                ContentType::JSON,
                make_bad_json_data_response(
                    "Could not add book to repo: bad path".to_string(),
                ),
            ),
        )
    }
}

/// *`POST /delete/<repo_path>`*
///
/// Typically mounted as **`/git/delete/<repo_path>`**
///
/// Deletes a local repo from the given repo path.
#[post("/delete/<repo_path..>")]
pub async fn delete_repo(
    state: &State<AppSettings>,
    repo_path: PathBuf,
) -> status::Custom<(ContentType, String)> {
    let path_components: Components<'_> = repo_path.components();
    if check_path_components(&mut path_components.clone()) {
        let path_to_delete = state.repo_dir.lock().unwrap().clone()
            + os_slash_str()
            + &repo_path.display().to_string();
        match std::fs::remove_dir_all(path_to_delete) {
            Ok(_) => status::Custom(
                Status::Ok,
                (
                    ContentType::JSON,
                    make_good_json_data_response("ok".to_string()),
                ),
            ),
            Err(e) => status::Custom(
                Status::BadRequest,
                (
                    ContentType::JSON,
                    make_bad_json_data_response(
                        format!("could not delete repo: {}", e).to_string(),
                    ),
                ),
            ),
        }
    } else {
        status::Custom(
            Status::BadRequest,
            (
                ContentType::JSON,
                make_bad_json_data_response("bad repo path".to_string()),
            ),
        )
    }
}

/// *`GET /status/<repo_path>`*
///
/// Typically mounted as **`/git/status/<repo_path>`**
///
/// Returns an array of changes to the local repo from the given repo path.
///
/// ```text
/// [
///   {
///     "path": "ingredients/LICENSE.md",
///     "change_type": "modified"
///   }
/// ]
/// ```
#[get("/status/<repo_path..>")]
pub async fn git_status(
    state: &State<AppSettings>,
    repo_path: PathBuf,
) -> status::Custom<(ContentType, String)> {
    let repo_path_string: String = state.repo_dir.lock().unwrap().clone()
        + os_slash_str()
        + &repo_path.display().to_string().clone();
    match Repository::open(repo_path_string) {
        Ok(repo) => {
            if repo.is_bare() {
                return status::Custom(
                    Status::BadRequest,
                    (
                        ContentType::JSON,
                        make_bad_json_data_response("cannot get status of bare repo".to_string()),
                    ),
                );
            };
            let mut opts = StatusOptions::new();
            opts.include_untracked(true);
            match repo.statuses(Some(&mut opts)) {
                Ok(statuses) => {
                    let mut status_changes = Vec::new();
                    for entry in statuses
                        .iter()
                        .filter(|e| e.status() != git2::Status::CURRENT)
                    {
                        let i_status = match entry.status() {
                            s if s.contains(git2::Status::INDEX_NEW)
                                || s.contains(git2::Status::WT_NEW) =>
                                {
                                    "new"
                                }
                            s if s.contains(git2::Status::INDEX_MODIFIED)
                                || s.contains(git2::Status::WT_MODIFIED) =>
                                {
                                    "modified"
                                }
                            s if s.contains(git2::Status::INDEX_DELETED)
                                || s.contains(git2::Status::WT_DELETED) =>
                                {
                                    "deleted"
                                }
                            s if s.contains(git2::Status::INDEX_RENAMED)
                                || s.contains(git2::Status::WT_RENAMED) =>
                                {
                                    "renamed"
                                }
                            s if s.contains(git2::Status::INDEX_TYPECHANGE)
                                || s.contains(git2::Status::WT_TYPECHANGE) =>
                                {
                                    "type_change"
                                }
                            _ => "",
                        };

                        if entry.status().contains(git2::Status::IGNORED) {
                            continue;
                        }
                        status_changes.push(GitStatusRecord {
                            path: entry.path().unwrap().to_string(),
                            change_type: i_status.to_string(),
                        });
                        // println!("{} ({})", entry.path().unwrap(), istatus);
                    }
                    status::Custom(
                        Status::Ok,
                        (
                            ContentType::JSON,
                            serde_json::to_string_pretty(&status_changes).unwrap(),
                        ),
                    )
                }
                Err(e) => status::Custom(
                    Status::InternalServerError,
                    (
                        ContentType::JSON,
                        make_bad_json_data_response(
                            format!("could not get repo status: {}", e).to_string(),
                        ),
                    ),
                ),
            }
        }
        Err(e) => status::Custom(
            Status::InternalServerError,
            (
                ContentType::JSON,
                make_bad_json_data_response(format!("could not open repo: {}", e).to_string()),
            ),
        ),
    }
}
