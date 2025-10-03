use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::os_slash_str;
use crate::utils::response::{not_ok_json_response, ok_ok_json_response};
use git2::{Repository, RepositoryInitOptions};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{post, FromForm, State};
use copy_dir::copy_dir;
use rocket::serde::Deserialize;
use crate::utils::time::utc_now_timestamp_string;

#[derive(FromForm, Deserialize)]
pub struct NewObsContentForm {
    pub content_name: String,
    pub content_abbr: String,
    pub content_language_code: String,
    pub branch_name: Option<String>
}

/// *`POST /new-obs-resource`*
///
/// Typically mounted as **`/git/new-obs-resource`**
///
/// Creates a new, local obs repo. It requires the following fields as a JSON body:
/// - content_name (string)
/// - content_abbr (string)
/// - content_language_code
/// - branch_name (null or string)
#[post("/new-obs-resource", format = "json", data = "<json_form>")]
pub fn new_obs_resource_repo(
    state: &State<AppSettings>,
    json_form: Json<NewObsContentForm>,
) -> status::Custom<(ContentType, String)> {
    // Check template type exists
    let path_to_template = format!(
        "{}{}templates{}content_templates{}text_stories{}metadata.json",
        &state.app_resources_dir,
        os_slash_str(),
        os_slash_str(),
        os_slash_str(),
        os_slash_str(),
    );
    if !std::path::Path::new(&path_to_template).is_file() {
        return not_ok_json_response(
            Status::BadRequest,
            make_bad_json_data_response(format!(
                "Metadata template {} not found",
                path_to_template
            )),
        );
    }
    // Build path for new repo and parent
    let path_to_new_repo_parent = format!(
        "{}{}_local_{}_local_",
        state.repo_dir.lock().unwrap().clone(),
        os_slash_str(),
        os_slash_str(),
    );
    let path_to_new_repo = format!(
        "{}{}{}",
        path_to_new_repo_parent.clone(),
        os_slash_str(),
        json_form.content_abbr.clone()
    );
    // Check path doesn't already exist
    if std::path::Path::new(&path_to_new_repo).exists() {
        return not_ok_json_response(
            Status::BadRequest,
            make_bad_json_data_response(format!(
                "Local content called '{}' already exists",
                json_form.content_abbr
            )),
        );
    }
    // Make parents?
    match std::fs::create_dir_all(path_to_new_repo_parent) {
        Ok(_) => (),
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!(
                    "Could not create local content directories: {}",
                    e
                )),
            )
        }
    }
    // Init repo
    let final_new_branch_name = json_form.branch_name.clone().unwrap_or("master".to_string());
    let mut repo_options = RepositoryInitOptions::new();
    let repo_options2 = repo_options.initial_head(final_new_branch_name.as_str());
    let new_repo = match Repository::init_opts(&path_to_new_repo, &repo_options2) {
        Ok(repo) => repo,
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!("Could not create repo: {}", e)),
            )
        }
    };
    // Set up local user info
    let mut config = new_repo.config().unwrap();
    config
        .set_str("user.name", whoami::username().as_str())
        .unwrap();
    config
        .set_str(
            "user.email",
            format!("{}@localhost", whoami::username().as_str()).as_str(),
        )
        .unwrap();
    // Copy ingredients dir
    let path_to_ingredients_template = format!(
        "{}{}templates{}content_templates{}text_stories{}ingredients",
        &state.app_resources_dir,
        os_slash_str(),
        os_slash_str(),
        os_slash_str(),
        os_slash_str(),
    );
    let path_to_ingredients = format!("{}{}ingredients", path_to_new_repo, os_slash_str(),);
    match copy_dir(&path_to_ingredients_template, &path_to_ingredients) {
        Ok(_) => (),
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!(
                    "Could not copy ingredients directory for repo: {}",
                    e
                )),
            )
        }
    }

    // Copy gitignore file
    let path_to_gitignore_template = format!(
        "{}{}templates{}content_templates{}gitignore.txt",
        &state.app_resources_dir,
        os_slash_str(),
        os_slash_str(),
        os_slash_str(),
    );
    let gitignore_string = match std::fs::read_to_string(&path_to_gitignore_template) {
        Ok(v) => v,
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!(
                    "Could not load gitignore template as string: {}",
                    e
                )),
            )
        }
    };
    let path_to_repo_gitignore =
        format!("{}{}.gitignore", path_to_new_repo, os_slash_str(),);
    match std::fs::write(path_to_repo_gitignore, &gitignore_string) {
        Ok(_) => (),
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!(
                    "Could not write gitignore to repo: {}",
                    e
                )),
            )
        }
    }

    // Read and customize metadata
    let mut metadata_string = match std::fs::read_to_string(&path_to_template) {
        Ok(v) => v,
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!(
                    "Could not load metadata template as string: {}",
                    e
                )),
            )
        }
    };
    let now_time = utc_now_timestamp_string();
    metadata_string = metadata_string
        .replace("%%ABBR%%", json_form.content_abbr.as_str())
        .replace("%%CONTENT_NAME%%", json_form.content_name.as_str())
        .replace("%%CONTENT_NAME%%", json_form.content_name.as_str())
        .replace("%%CREATED_TIMESTAMP%%", now_time.to_string().as_str());

    // Write metadata
    let path_to_repo_metadata = format!("{}{}metadata.json", &path_to_new_repo, os_slash_str());
    match std::fs::write(path_to_repo_metadata, metadata_string) {
        Ok(_) => (),
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!(
                    "Could not write metadata template to repo: {}",
                    e
                )),
            )
        }
    }
    // Add and commit
    new_repo
        .index()
        .unwrap()
        .add_all(&["."], git2::IndexAddOption::DEFAULT, None)
        .unwrap();
    new_repo.index().unwrap().write().unwrap();
    let sig = new_repo.signature().unwrap();
    let tree_id = {
        let mut index = new_repo.index().unwrap();
        index.write_tree().unwrap()
    };
    let tree = new_repo.find_tree(tree_id).unwrap();
    new_repo
        .commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
        .unwrap();
    ok_ok_json_response()
}
