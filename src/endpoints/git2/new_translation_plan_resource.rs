use crate::structs::AppSettings;
use crate::utils::burrito::{copy_gitignore_template, copy_vrs_template, language_name_from_code};
use crate::utils::files::{paths_to_new_burrito};
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::os_slash_str;
use crate::utils::response::{not_ok_json_response, ok_ok_json_response};
use crate::utils::time::utc_now_timestamp_string;
use crate::utils::git::{init_repo, add_and_commit_repo};
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
/// - copyright (null or string)
/// - content_language_code
/// - content_language_name (null or string)
/// - versification (null or string)
/// - branch_name(null or string)
/// - plan (null or string)

#[derive(FromForm, Deserialize)]
pub struct NewTranslationPlanContentForm {
    pub content_name: String,
    pub content_abbr: String,
    pub copyright: Option<String>,
    pub content_language_code: String,
    pub content_language_name: Option<String>,
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
    let language_name = match language_name_from_code(
        &state.app_resources_dir,
        json_form.content_language_code.clone(),
        json_form.content_language_name.clone(),
    ) {
        Ok(v) => v,
        Err(e) => {
            return not_ok_json_response(
                Status::BadRequest,
                make_bad_json_data_response(format!(
                    "Unable to find language name: {}: {}",
                    &json_form.content_language_code, e
                )),
            );
        }
    };

    let (path_to_new_repo_parent, path_to_new_repo) =
        match paths_to_new_burrito(&state.repo_dir.lock().unwrap(), &json_form.content_abbr) {
            Ok(tup) => tup,
            Err(e) => {
                return not_ok_json_response(
                    Status::BadRequest,
                    make_bad_json_data_response(format!("Could not get new repo paths: {}", e)),
                )
            }
        };

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
    let new_repo = match init_repo(&path_to_new_repo, json_form.branch_name.clone()) {
        Ok(r) => r,
        Err(e) => {
            return status::Custom(
                Status::InternalServerError,
                (
                    ContentType::JSON,
                    make_bad_json_data_response(format!(
                        "{}",
                        e
                    )),
                ),
            )
        }
    };
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

    match copy_gitignore_template(&state.app_resources_dir, &path_to_new_repo) {
        Ok(_) => {},
        Err(e) => return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!("{}", e)),
            )
    };

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
        .replace(
            "%%COPYRIGHT%%",
            json_form
                .copyright
                .clone()
                .unwrap_or("unspecified".to_string())
                .as_str(),
        )
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
            "tag": &json_form.content_language_code,
            "name": {
                "en": &language_name,
        }
        }
    );
    metadata_string = metadata_string
        .replace("%%ABBR%%", json_form.content_abbr.as_str())
        .replace("%%CONTENT_NAME%%", json_form.content_name.as_str())
        .replace(
            "%%COPYRIGHT%%",
            json_form
                .copyright
                .clone()
                .unwrap_or("unspecified".to_string())
                .as_str(),
        )
        .replace("%%CREATED_TIMESTAMP%%", now_time.to_string().as_str())
        .replace(
            "%%LANGUAGE%%",
            serde_json::to_string(&language_json)
                .expect("language json")
                .as_str(),
        );
    match copy_vrs_template(
        &state.app_resources_dir,
        &path_to_new_repo,
        &json_form.versification.clone().unwrap_or("eng".to_string()),
    ) {
        Ok(_) => {}
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!("{}", e)),
            )
        }
    };
    
    // Ingredients from plan
    let mut plan_books = std::collections::BTreeSet::new();
    let translation_plan_value =
        serde_json::from_str::<Value>(&plan_template_string).expect("plan from str");
    let translation_plan_object = translation_plan_value.as_object().expect("plan as object");
    let translation_plan_sections = translation_plan_object["sections"]
        .as_array()
        .expect("plan sections as array")
        .to_vec();
    for section in translation_plan_sections.iter() {
        let book_code = section["bookCode"]
            .as_str()
            .expect("bookCode as string")
            .to_string();
        plan_books.insert(book_code);
    }
    let scope_string = plan_books
        .iter()
        .map(|b| format!("\"{}\": {{}}", b))
        .collect::<Vec<_>>()
        .join(", ");
    metadata_string = metadata_string.replace("%%SCOPE%%", &scope_string);
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
    match add_and_commit_repo(new_repo, &"Initial Commit".to_string()) {
        Ok(_) => {},
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!(
                    "{}",
                    e
                )),
            )
        }
    };
    ok_ok_json_response()
}
