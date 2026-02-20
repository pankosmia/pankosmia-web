use std::path::Path;

use crate::structs::AppSettings;
use crate::utils::paths::os_slash_str;
use crate::utils::response::{ok_json_response, not_ok_json_response};
use crate::utils::json_responses::make_bad_json_data_response;
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{post, State};
use serde::Deserialize;
use serde_json::{json, Value};
use regex::Regex;

#[derive(Deserialize)]
pub struct RagPromptForm {
    model_name: String,
    quantized: bool,
    book: String,
    chapter: u8,
    from_verse: u8,
    to_verse: Option<u8>,
    rag_context: Value,
}

/// *`POST /rag-prompt`*
///
/// Typically mounted as **`/llm/rag-prompt`**
///
/// Builds a RAG prompt, processes it and returns the result
#[post("/rag-prompt", format = "json", data = "<form>")]
pub async fn post_rag_prompt(
    state: &State<AppSettings>,
    form: Json<RagPromptForm>,
) -> status::Custom<(ContentType, String)> {
    let safe_field_regex = Regex::new(r"[^A-Za-z0-9-]").unwrap();

    let safe_field = safe_field_regex.replace(form.model_name.as_str(), "_");
    let working_path = state.working_dir.clone();
    let model_suffix;
    if form.quantized {
        model_suffix = ".quant";
    } else {
        model_suffix = "";
    }
    let model_path = format!(
        "{}{}blobs{}llm_models{}{}{}model{}.onnx",
        &working_path,
        os_slash_str(),
        os_slash_str(),
        os_slash_str(),
        &safe_field,
        os_slash_str(),
        &model_suffix
    );
    if !Path::new(&model_path).exists() {
        return not_ok_json_response(
            Status::BadRequest,
            make_bad_json_data_response(
                format!("Could not find model '{}'", &safe_field)
            )
        );
    }
    ok_json_response(json!({}).to_string())
}
