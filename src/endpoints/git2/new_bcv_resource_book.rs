use crate::structs::{AppSettings, BurritoMetadata, NewBcvResourceBookForm};
use crate::utils::burrito::{ingredients_metadata_from_files, ingredients_scopes_from_files};
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::{check_local_path_components, os_slash_str};
use crate::utils::response::{
    not_ok_bad_repo_json_response, not_ok_json_response, ok_ok_json_response,
};
use copy_dir::copy_dir;
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{post, State};
use std::path::{Components, Path, PathBuf};

/// *`POST /new-scripture-book/<repo_path>`*
///
/// Typically mounted as **`/git/new-scripture-book/<repo_path>`**
///
/// Adds a Scripture book to a local repo at the given repo path.
///
///  It requires the following fields as a JSON body:
/// - book_code (string)
/// - book_title (string)
/// - book_abbr (string)
/// - add_cv (boolean)
#[allow(irrefutable_let_patterns)]
#[post(
    "/new-bcv-resource-book/<repo_path..>",
    format = "json",
    data = "<json_form>"
)]
pub async fn new_bcv_resource_book(
    state: &State<AppSettings>,
    repo_path: PathBuf,
    json_form: Json<NewBcvResourceBookForm>,
) -> status::Custom<(ContentType, String)> {
    let path_components: Components<'_> = repo_path.components();
    if check_local_path_components(&mut path_components.clone()) {
        let app_resources_dir = format!("{}", &state.app_resources_dir);
        // Read metadata
        let repo_dir_path = state.repo_dir.lock().unwrap().clone();
        let repo_name = path_components
            .skip(2)
            .next()
            .unwrap()
            .as_os_str()
            .to_str()
            .unwrap();
        let full_repo_dir = format!(
            "{}{}_local_{}_local_{}{}",
            repo_dir_path,
            os_slash_str(),
            os_slash_str(),
            os_slash_str(),
            &repo_name
        );
        let path_to_repo_metadata = format!("{}{}metadata.json", full_repo_dir, os_slash_str(),);
        let metadata_string = match std::fs::read_to_string(&path_to_repo_metadata) {
            Ok(v) => v,
            Err(e) => {
                return not_ok_json_response(
                    Status::InternalServerError,
                    make_bad_json_data_response(format!(
                        "Could not load metadata as string: {}",
                        e
                    )),
                )
            }
        };
        // Make struct from metadata
        let mut metadata_struct: BurritoMetadata = match serde_json::from_str(&metadata_string) {
            Ok(v) => v,
            Err(e) => {
                return not_ok_json_response(
                    Status::InternalServerError,
                    make_bad_json_data_response(format!("Could not parse metadata: {}", e)),
                );
            }
        };
        // Check new book isn't already there
        let new_ingredients_path = format!("ingredients/{}.tsv", &json_form.book_code);
        if let ingredients = metadata_struct.ingredients.lock().unwrap() {
            if ingredients.contains_key(&new_ingredients_path) {
                return not_ok_json_response(
                    Status::BadRequest,
                    make_bad_json_data_response(format!(
                        "Book '{}' already exists in metadata",
                        &json_form.book_code
                    )),
                );
            }
        }
        // TODO Make empty book TSV
        // TODO Find flavor so we can load correct template
        let type_info = metadata_struct.r#type.clone();
        let type_ob = type_info.as_object().expect("Metadata type as object");
        let flavor_type_ob = type_ob["flavorType"].as_object().expect("flavorType as object");
        let flavor_ob = flavor_type_ob["flavor"].as_object().expect("Flavor as object");
        let flavor_string = flavor_ob["name"].as_str().expect("Flavor name as string").to_lowercase();
        let flavor_abbreviation = if flavor_string == "x-bcvnotes" {
            "tn"
        } else if flavor_string == "x-bcvquestions" {
            "tn"
        } else if flavor_string == "x-studyquestions" {
            "sq"
        } else {"tq"};

        let path_to_tsv_template = format!(
            "{}{}app_resources{}tsv{}{}.tsv",
            &state.app_resources_dir,
            os_slash_str(),
            os_slash_str(),
            os_slash_str(),
            flavor_abbreviation
        );
        let path_to_new_book = format!(
            "{}{}_local_{}_local_{}{}{}ingredients{}{}.tsv",
            repo_dir_path,
            os_slash_str(),
            os_slash_str(),
            os_slash_str(),
            &repo_name,
            os_slash_str(),
            os_slash_str(),
            &json_form.book_code
        );
        match copy_dir(&path_to_tsv_template, &path_to_new_book) {
            Ok(_) => {}
            Err(e) => {
                return not_ok_json_response(
                    Status::BadRequest,
                    make_bad_json_data_response(format!("could not copy new book from template to repo: {}", e).to_string()),
                )
            }
        }
        // Path to VRS file in repo
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
        // Does that repo VRS file exist?
        let repo_vrs_found = Path::new(&path_to_repo_vrs).is_file();
        if !repo_vrs_found {
            // If not -
            // -- do we have a vrs_name value? If not die
            let vrs_name = match json_form.vrs_name.clone() {
                Some(v) => v,
                None => {
                    return not_ok_json_response(
                        Status::InternalServerError,
                        make_bad_json_data_response("No VRS ingredient found in repo and no versification name provided in API call".to_string()),
                    )
                }
            };
            //    Get versification file as JSON
            let path_to_template_vrs = format!(
                "{}{}templates{}content_templates{}vrs{}{}.json",
                &state.app_resources_dir,
                os_slash_str(),
                os_slash_str(),
                os_slash_str(),
                os_slash_str(),
                json_form.vrs_name.clone().unwrap(),
            );
            // -- is there a template file for that vrs_name? If not die
            let template_vrs_found = Path::new(&path_to_template_vrs).is_file();
            if !template_vrs_found {
                return not_ok_json_response(
                    Status::InternalServerError,
                    make_bad_json_data_response(format!("No VRS template for {} found in repo and no versification name provided in API call", vrs_name.clone())),
                );
            };
            // -- copy from template to repo, and we're good to go
            match copy_dir(&path_to_template_vrs, &path_to_repo_vrs) {
                Ok(_) => {}
                Err(e) => {
                    return not_ok_json_response(
                        Status::BadRequest,
                        make_bad_json_data_response(
                            format!("could not copy vrs from template to repo: {}", e).to_string(),
                        ),
                    )
                }
            }
        }
        // Add ingredient record and currentScope value for TSV
        if let mut ingredients = metadata_struct.ingredients.lock().unwrap() {
            let new_ingredients = ingredients_metadata_from_files(app_resources_dir.clone(), full_repo_dir.clone());
            *ingredients = new_ingredients;
        }
        if let type_info = metadata_struct.r#type {
            let mut type_ob = type_info.as_object().unwrap().clone();
            let flavor_type_ob = type_ob["flavorType"].as_object_mut().unwrap();
            let new_current_scope = ingredients_scopes_from_files(app_resources_dir, full_repo_dir.clone());
            flavor_type_ob["currentScope"] =
                serde_json::from_str(serde_json::to_string(&new_current_scope).unwrap().as_str())
                    .unwrap();
            metadata_struct.r#type =
                serde_json::from_str(serde_json::to_string(&type_ob).unwrap().as_str()).unwrap();
        }

        // Write metadata
        let metadata_output_string = match serde_json::to_string(&metadata_struct) {
            Ok(s) => s,
            Err(e) => {
                return not_ok_json_response(
                    Status::InternalServerError,
                    make_bad_json_data_response(format!("Could not make metadata as JSON: {}", e)),
                )
            }
        };
        match std::fs::write(path_to_repo_metadata, &metadata_output_string) {
            Ok(_) => (),
            Err(e) => {
                return not_ok_json_response(
                    Status::InternalServerError,
                    make_bad_json_data_response(format!("Could not write metadata to repo: {}", e)),
                )
            }
        }
        // Add and commit
        ok_ok_json_response()
    } else {
        not_ok_bad_repo_json_response()
    }
}
