use crate::structs::AppSettings;
use crate::utils::files::load_json;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::os_slash_str;
use crate::utils::response::{not_ok_json_response, ok_ok_json_response};
use crate::utils::time::utc_now_timestamp_string;
use git2::{Repository, RepositoryInitOptions};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use rocket::{post, FromForm, State};
use serde_json;
use serde_json::{json, Value};

/// *`POST /new-translation-plan-resource`*
///
/// Typically mounted as **`/git/new-translation-plan-resource`**
///
/// Creates a new, local x-translationplan* repo. It requires the following fields as a JSON body:
/// - content_name (string)
/// - content_abbr (string)
/// - copyright (string)
/// - content_language_code
/// - versification (null or string)
/// - branch_name(null or string)
/// - plan (null or string)

#[derive(FromForm, Deserialize)]
pub struct NewTranslationPlanContentForm {
    pub content_name: String,
    pub content_abbr: String,
    pub copyright: String,
    pub content_language_code: String,
    pub versification: Option<String>,
    pub branch_name: Option<String>,
    pub plan: Option<String>,
}

#[post(
    "/new-translation-plan-resource",
    format = "json",
    data = "<json_form>"
)]
pub fn new_translation_plan_resource_repo(
    state: &State<AppSettings>,
    json_form: Json<NewTranslationPlanContentForm>,
) -> status::Custom<(ContentType, String)> {
    // Check template type exists
    let path_to_template = format!(
        "{}{}templates{}content_templates{}x-translationplan{}metadata.json",
        &state.app_resources_dir,
        os_slash_str(),
        os_slash_str(),
        os_slash_str(),
        os_slash_str(),
    );
    if !std::path::Path::new(&path_to_template).is_file() {
        return not_ok_json_response(
            Status::BadRequest,
            make_bad_json_data_response(format!("Metadata template not found")),
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
    let final_new_branch_name = json_form.branch_name.clone().unwrap_or("main".to_string());
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
    // Make ingredients dir
    let path_to_ingredients = format!("{}{}ingredients", path_to_new_repo, os_slash_str(),);
    match std::fs::create_dir(&path_to_ingredients) {
        Ok(_) => (),
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!(
                    "Could not create ingredients directory for repo: {}",
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
    let path_to_repo_gitignore = format!("{}{}.gitignore", path_to_new_repo, os_slash_str(),);
    match std::fs::write(path_to_repo_gitignore, &gitignore_string) {
        Ok(_) => (),
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!("Could not write gitignore to repo: {}", e)),
            )
        }
    }

    // Use default plan template or make supplied plan into a template
    let path_to_repo_plan = format!(
        "{}{}ingredients{}plan.json",
        path_to_new_repo,
        os_slash_str(),
        os_slash_str()
    );
    let mut plan_template_string = match json_form.plan.clone() {
        Some(p) => {
            let plan_value: Value = match serde_json::from_str(&p) {
                Ok(pv) => pv,
                Err(e) => {
                    return not_ok_json_response(
                        Status::InternalServerError,
                        make_bad_json_data_response(format!(
                            "Could not read supplied plan as JSON: {}",
                            e
                        )),
                    )
                }
            };
            let mut plan_value_object: serde_json::Map<String, Value> = match plan_value.as_object()
            {
                Some(pvo) => pvo.clone(),
                None => {
                    return not_ok_json_response(
                        Status::InternalServerError,
                        make_bad_json_data_response(
                            "Could not treat supplied plan as map".to_string(),
                        ),
                    )
                }
            };
            plan_value_object.insert(
                "name".to_string(),
                Value::String("%%CONTENTNAME%%".to_string()),
            );
            plan_value_object.insert(
                "description".to_string(),
                Value::String("%%CONTENTDESCRIPTION%%".to_string()),
            );
            plan_value_object.insert(
                "copyright".to_string(),
                Value::String("%%COPYRIGHT%%".to_string()),
            );
            plan_value_object.insert(
                "short_name".to_string(),
                Value::String("%%CONTENTABBR%%".to_string()),
            );
            plan_value_object.insert(
                "versification".to_string(),
                Value::String("%%CONTENTVERSIFICATION%%".to_string()),
            );
            plan_value_object.insert(
                "version".to_string(),
                Value::String("%%VERSION%%".to_string()),
            );
            serde_json::to_string(&plan_value_object).expect("plan map to string")
        }
        None => {
            let path_to_plan_template = format!(
                "{}{}templates{}content_templates{}x-translationplan{}plan.json",
                &state.app_resources_dir,
                os_slash_str(),
                os_slash_str(),
                os_slash_str(),
                os_slash_str(),
            );
            match std::fs::read_to_string(&path_to_plan_template) {
                Ok(v) => v,
                Err(e) => {
                    return not_ok_json_response(
                        Status::InternalServerError,
                        make_bad_json_data_response(format!(
                            "Could not load translation plan template as string: {}",
                            e
                        )),
                    )
                }
            }
        }
    };
    // Substitute into plan template
    plan_template_string = plan_template_string
        .replace("%%CONTENTNAME%%", json_form.content_name.as_str())
        .replace("%%CONTENTDESCRIPTION%%", json_form.content_name.as_str())
        .replace("%%COPYRIGHT%%", json_form.copyright.as_str())
        .replace("%%CONTENTABBR%%", json_form.content_abbr.as_str())
        .replace(
            "%%CONTENTVERSIFICATION%%",
            json_form
                .versification
                .clone()
                .unwrap_or("org".to_string())
                .as_str(),
        )
        .replace("%%VERSION%%", "0.0.1");
    match std::fs::write(path_to_repo_plan, &plan_template_string) {
        Ok(_) => (),
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!(
                    "Could not write translation plan template to repo: {}",
                    e
                )),
            )
        }
    }

    // Read language lookup
    let path_to_language_lookup = format!(
        "{}{}app_resources{}lookups{}bcp47-language_codes.json",
        &state.app_resources_dir,
        os_slash_str(),
        os_slash_str(),
        os_slash_str(),
    );

    let language_lookup_json = match load_json(&path_to_language_lookup) {
        Ok(v) => v,
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!(
                    "Could not load and parse language lookup: {}",
                    e
                )),
            )
        }
    };

    let language_tag = match language_lookup_json[&json_form.content_language_code].as_object() {
        Some(_) => json_form.content_language_code.clone(),
        None => format!("x-{}", &json_form.content_language_code),
    };

    let language_name = match language_lookup_json[&json_form.content_language_code].as_object() {
        Some(r) => r["en"].as_str().expect("English language name").to_string(),
        None => json_form.content_language_code.clone(),
    };

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
    let language_json = json!(
        {
            "tag": &language_tag,
            "name": {
                "en": &language_name,
        }
        }
    );
    metadata_string = metadata_string
        .replace("%%ABBR%%", json_form.content_abbr.as_str())
        .replace("%%CONTENT_NAME%%", json_form.content_name.as_str())
        .replace("%%COPYRIGHT%%", json_form.copyright.as_str())
        .replace("%%CREATED_TIMESTAMP%%", now_time.to_string().as_str())
        .replace(
            "%%LANGUAGE%%",
            serde_json::to_string(&language_json)
                .expect("language json")
                .as_str(),
        );
    // Get versification file as JSON
    let path_to_versification = format!(
        "{}{}templates{}content_templates{}vrs{}{}.json",
        &state.app_resources_dir,
        os_slash_str(),
        os_slash_str(),
        os_slash_str(),
        os_slash_str(),
        json_form.versification.clone().unwrap_or("eng".to_string()),
    );
    let versification_schema = match load_json(&path_to_versification) {
        Ok(j) => j,
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!("Could not load versification JSON: {}", e)),
            )
        }
    };
    // Write it out to new repo
    let path_to_repo_versification =
        format!("{}{}ingredients/vrs.json", path_to_new_repo, os_slash_str(),);
    let versification_string = serde_json::to_string(&versification_schema).unwrap();
    match std::fs::write(path_to_repo_versification, &versification_string) {
        Ok(_) => (),
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!(
                    "Could not write versification to repo: {}",
                    e
                )),
            )
        }
    }

    // No ingredients
    metadata_string = metadata_string.replace("%%SCOPE%%", "");
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
