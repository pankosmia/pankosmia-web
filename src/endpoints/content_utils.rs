use rocket::{get, State};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::os_slash_str;

// **TEMPLATES**
/// *`GET /templates`*
///
/// Typically mounted as **`/content-utils/templates`**
///
/// Returns a JSON array of local content template names.
///
/// `["text_translation"]`
#[get("/templates")]
pub fn list_content_templates(state: &State<AppSettings>) -> status::Custom<(ContentType, String)> {
    let root_path = state.app_resources_dir.clone();
    let templates_dir = format!("{}{}{}{}{}", root_path, os_slash_str(), "templates", os_slash_str(), "content_templates");
    let template_paths = std::fs::read_dir(templates_dir).unwrap();
    let mut templates: Vec<String> = Vec::new();
    for template_path in template_paths {
        let template_path_ob = template_path.unwrap().path();
        let template_filename = template_path_ob.file_name().unwrap();
        templates.push(template_filename.to_str().unwrap().to_string().split(".").next().unwrap().to_string());
    }
    let content_json_string = serde_json::to_string_pretty(&templates).unwrap();
    status::Custom(
        Status::Ok,
        (
            ContentType::JSON,
            content_json_string,
        ),
    )
}

/// *`GET /metadata-template/<template_name>`*
///
/// Typically mounted as **`/content-utils/metadata-template/<template_name>`**
///
/// Returns a metadata content template as JSON
#[get("/metadata-template/<template_name>")]
pub async fn content_metadata_template(
    state: &State<AppSettings>,
    template_name: String,
) -> status::Custom<(ContentType, String)> {
    let path_to_serve = format!(
        "{}{}{}{}{}{}{}{}{}",
        &state.app_resources_dir,
        os_slash_str(),
        "templates",
        os_slash_str(),
        "content_templates",
        os_slash_str(),
        template_name.clone(),
        os_slash_str(),
        "metadata.json"
    );

    match std::fs::read_to_string(path_to_serve) {
        Ok(v) => {
            status::Custom(
                Status::Ok,
                (
                    ContentType::JSON,
                    v,
                ),
            )
        }
        Err(e) => status::Custom(
            Status::BadRequest,
            (
                ContentType::JSON,
                make_bad_json_data_response(
                    format!("could not read content metadata template '{}': {}", template_name, e).to_string(),
                ),
            ),
        ),
    }
}

/// *`GET /template-filenames/<template>`*
///
/// Typically mounted as **`/content-utils/template-filenames/<templates>`**
///
/// Returns a JSON array of local content files for a given content template.
///
/// `["book.usfm", "metadata.json"]`
#[get("/template-filenames/<template>")]
pub fn list_content_template_filenames(
    state: &State<AppSettings>,
    template: String,
) -> status::Custom<(ContentType, String)> {
    let root_path = state.app_resources_dir.clone();
    let template_dir = format!(
        "{}{}{}{}{}{}{}",
        root_path,
        os_slash_str(),
        "templates",
        os_slash_str(),
        "content_templates",
        os_slash_str(),
        template.clone()
    );
    let filename_paths = std::fs::read_dir(template_dir).unwrap();
    let mut filenames: Vec<String> = Vec::new();
    for filename_path in filename_paths {
        let filename_path_ob = filename_path.unwrap().path();
        let filename_filename = filename_path_ob.file_name().unwrap();
        filenames.push(filename_filename.to_str().unwrap().to_string());
    }
    let content_json_string = serde_json::to_string_pretty(&filenames).unwrap();
    status::Custom(
        Status::Ok,
        (
            ContentType::JSON,
            content_json_string,
        ),
    )
}

/// *`GET /template/<template_name>/<filename>`*
///
/// Typically mounted as **`/content-utils/template/<template_name>/<filename>`**
///
/// Returns a content template of a particular type as plain text. The filename includes the suffix.
#[get("/template/<template_name>/<filename>")]
pub async fn content_template(
    state: &State<AppSettings>,
    template_name: String,
    filename: String
) -> status::Custom<(ContentType, String)> {
    let path_to_serve = format!(
        "{}{}{}{}{}{}{}{}{}",
        &state.app_resources_dir,
        os_slash_str(),
        "templates",
        os_slash_str(),
        "content_templates",
        os_slash_str(),
        template_name.clone(),
        os_slash_str(),
        filename.clone()
    );

    match std::fs::read_to_string(path_to_serve) {
        Ok(v) => {
            status::Custom(
                Status::Ok,
                (
                    ContentType::Plain,
                    v,
                ),
            )
        }
        Err(e) => status::Custom(
            Status::BadRequest,
            (
                ContentType::JSON,
                make_bad_json_data_response(
                    format!(
                        "could not read file {} for content template '{}': {}",
                        filename,
                        template_name,
                        e
                    ).to_string(),
                ),
            ),
        ),
    }
}

// **VERSIFICATION**
/// *`GET /versifications`*
///
/// Typically mounted as **`/content-utils/versifications`**
///
/// Returns a JSON array of versification schemes.
///
/// `["ENG", "ORG", "LXX"]`
#[get("/versifications")]
pub fn list_versifications(state: &State<AppSettings>) -> status::Custom<(ContentType, String)> {
    let root_path = state.app_resources_dir.clone();
    let versification_dir = format!(
        "{}{}{}{}{}{}{}",
        root_path,
        os_slash_str(),
        "templates",
        os_slash_str(),
        "content_templates",
        os_slash_str(),
        "vrs"
    );
    let versification_paths = std::fs::read_dir(versification_dir).unwrap();
    let mut versifications: Vec<String> = Vec::new();
    for versification_path in versification_paths {
        let versification_path_ob = versification_path.unwrap().path();
        let versification_filename = versification_path_ob.file_name().unwrap();
        versifications.push(versification_filename.to_str().unwrap().to_string().split(".").next().unwrap().to_string());
    }
    let content_json_string = serde_json::to_string_pretty(&versifications).unwrap();
    status::Custom(
        Status::Ok,
        (
            ContentType::JSON,
            content_json_string,
        ),
    )
}

/// *`GET /versification/<versification_name>`*
///
/// Typically mounted as **`/content-utils/versification/<versification_name>`**
///
/// Returns chapter/verse info for a given versification as JSON
#[get("/versification/<versification_name>")]
pub async fn versification(
    state: &State<AppSettings>,
    versification_name: String,
) -> status::Custom<(ContentType, String)> {
    let path_to_serve = format!(
        "{}{}{}{}{}{}{}{}{}",
        &state.app_resources_dir,
        os_slash_str(),
        "templates",
        os_slash_str(),
        "content_templates",
        os_slash_str(),
        "vrs",
        os_slash_str(),
        versification_name.clone() + ".json"
    );

    match std::fs::read_to_string(path_to_serve) {
        Ok(v) => {
            status::Custom(
                Status::Ok,
                (
                    ContentType::JSON,
                    v,
                ),
            )
        }
        Err(e) => status::Custom(
            Status::BadRequest,
            (
                ContentType::JSON,
                make_bad_json_data_response(
                    format!("could not read versification file for '{}': {}", versification_name, e).to_string(),
                ),
            ),
        ),
    }
}
