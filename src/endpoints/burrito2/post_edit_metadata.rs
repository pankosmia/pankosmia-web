use crate::structs::{AppSettings, BurritoMetadata, EditMetadataRecord};
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::{check_path_components, os_slash_str};
use crate::utils::response::{
    not_ok_bad_repo_json_response, not_ok_json_response, ok_ok_json_response,
};
use crate::utils::burrito::language_name_from_code;
use crate::utils::time::utc_now_timestamp_string;
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{post, State};
use serde_json::{Map, Value};
use std::path::{Components, PathBuf};
use uuid::Uuid;

/// *`POST /metadata/edit/<repo_path>`*
///
/// Typically mounted as **`/api/burrito/metadata/edit/<repo_path>`**
///
/// Edits the Dublin Core-like parts of the metadata for a repo.
/// The data is provided as a JSON payload:
/// {
///    "name": "NEW NAME",
///    "description": "THIS IS A NEW DESCRIPTION",
///    "abbreviation": "NN",
///    "language_code": "x-foo",
///    "language_name": "Green Fooish",
///    "script_direction": "rtl",
///    "copyright": "© The Foo Fairies"
/// }

#[post("/metadata/edit/<repo_path..>", format = "json", data = "<edits>")]
#[allow(irrefutable_let_patterns)]
pub async fn post_edit_metadata(
    state: &State<AppSettings>,
    repo_path: PathBuf,
    edits: Json<EditMetadataRecord>,
) -> status::Custom<(ContentType, String)> {
    let path_components: Components<'_> = repo_path.components();
    let full_repo_path = format!(
        "{}{}{}",
        state.repo_dir.lock().expect("repo dir lock"),
        os_slash_str(),
        &repo_path.display().to_string()
    );
    if check_path_components(&mut path_components.clone())
        && std::fs::metadata(&full_repo_path).is_ok()
    {
        // Get metadata as string
        let path_to_repo_metadata = format!("{}{}metadata.json", &full_repo_path, os_slash_str(),);
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
        // Make struct from metadata string
        let mut metadata_struct: BurritoMetadata = match serde_json::from_str(&metadata_string) {
            Ok(v) => v,
            Err(e) => {
                return not_ok_json_response(
                    Status::InternalServerError,
                    make_bad_json_data_response(format!("Could not parse metadata: {}", e)),
                );
            }
        };

        // Update timestamps, revision
        let now_time = utc_now_timestamp_string();
        // - meta timestamp
        let mut meta_obj: Map<String, Value> = metadata_struct
            .meta
            .as_object()
            .expect("meta object")
            .clone();
        meta_obj.insert("dateCreated".to_string(), Value::String(now_time.clone()));
        metadata_struct.meta = serde_json::to_value(meta_obj).expect("meta map to value");
        // - find identification abbr object (needed to find timestamp and revision)
        let mut identification_obj: Map<String, Value> = metadata_struct
            .identification
            .as_object()
            .expect("identification as object")
            .clone();
        let mut primary_obj = identification_obj["primary"]
            .as_object()
            .expect("primary as object")
            .clone();
        let first_primary_org_tuple = primary_obj.iter().next().expect("first org");
        let first_primary_org_key = first_primary_org_tuple.0;
        let mut first_primary_org = first_primary_org_tuple
            .1
            .as_object()
            .expect("org object")
            .clone();
        let first_primary_abbr_tuple = first_primary_org.iter().next().expect("first abbr");
        let first_primary_abbr_key = first_primary_abbr_tuple.0;
        let mut first_primary_abbr = first_primary_abbr_tuple
            .1
            .as_object()
            .expect("abbr object")
            .clone();
        let new_revision = Uuid::new_v4().to_string();
        first_primary_abbr.insert("revision".to_string(), Value::String(new_revision));
        // - identification timestamp
        first_primary_abbr.insert("timestamp".to_string(), Value::String(now_time));

        // Update primary in identification
        first_primary_org.insert(
            first_primary_abbr_key.clone(),
            serde_json::to_value(first_primary_abbr).expect("abbr value"),
        );
        primary_obj.insert(
            first_primary_org_key.clone(),
            serde_json::to_value(first_primary_org).expect("org value"),
        );
        identification_obj.insert("primary".to_string(), Value::Object(primary_obj));

        // Edit identity info (name, description, abbreviation)
        // Directly in identification as lang/text tuples
        let mut name_obj = identification_obj["name"]
            .as_object()
            .expect("identification name as object")
            .clone();
        name_obj.insert("en".to_string(), Value::String(edits.name.clone()));
        identification_obj.insert("name".to_string(), Value::Object(name_obj));

        let mut description_obj = identification_obj["description"]
            .as_object()
            .expect("identification description as object")
            .clone();
        description_obj.insert("en".to_string(), Value::String(edits.description.clone()));
        identification_obj.insert("description".to_string(), Value::Object(description_obj));

        let mut abbreviation_obj = identification_obj["abbreviation"]
            .as_object()
            .expect("identification abbreviation as object")
            .clone();
        abbreviation_obj.insert("en".to_string(), Value::String(edits.abbreviation.clone()));
        identification_obj.insert("abbreviation".to_string(), Value::Object(abbreviation_obj));

        // Update identification
        metadata_struct.identification =
            serde_json::to_value(identification_obj).expect("new identification");

        // Edit language (code => tag, name => BTreeMap, scriptDirection => Option)
        // Need if no language_name, look up from code
        let language_name = match language_name_from_code(
            &state.app_resources_dir,
            edits.language_code.clone(),
            edits.language_name.clone(),
        ) {
            Ok(ln) => ln,
            Err(e) => return not_ok_json_response(
                    Status::BadRequest,
                    make_bad_json_data_response(format!("Could not read or infer language name: {}", e)),
                )
        };
        metadata_struct.languages[0].tag = edits.language_code.clone();
        metadata_struct.languages[0]
            .name
            .lock()
            .expect("lock on language name")
            .insert("en".to_string(), language_name);
        metadata_struct.languages[0].scriptDirection = Some(edits.script_direction.clone());

        // Edit copyright
        let mut short_statements_array = metadata_struct.copyright["shortStatements"].as_array().expect("shortStatements array").clone();
        let mut short_statement = short_statements_array[0].as_object_mut().expect("shortStatement object").clone();
        short_statement.insert("en".to_string(), Value::from(edits.copyright.clone()));
        short_statements_array[0] = Value::from(short_statement);
        metadata_struct.copyright["shortStatements"] = Value::from(short_statements_array);

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
        ok_ok_json_response()
    } else {
        not_ok_bad_repo_json_response()
    }
}
