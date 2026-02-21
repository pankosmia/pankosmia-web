use std::path::Path;

use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::os_slash_str;
use crate::utils::response::{not_ok_json_response, ok_json_response};
use pankosmia_rag_chat::{do_one_iteration, generator_from_model, VerseContext};
use regex::Regex;
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{post, State};
use rten::Model;
use rten_text::Tokenizer;
use serde::Deserialize;
use serde_json::json;
use std::time::Instant;

#[derive(Deserialize)]
pub struct RagPromptForm {
    model_name: String,
    quantized: bool,
    book: String,
    from_chapter: u8,
    to_chapter: Option<u8>,
    from_verse: u8,
    to_verse: Option<u8>,
    rag_context: VerseContext,
    top_k: usize,
    temperature: f32,
    prompt: String,
    show_prompt: bool
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
    let safe_field_regex = Regex::new(r"[^A-Za-z0-9._-]").unwrap();
    let safe_model_name = safe_field_regex.replace(form.model_name.as_str(), "_");
    let working_path = state.working_dir.clone();

    // Check model exists
    let model_dir_path = format!(
        "{}{}blobs{}llm_models{}{}",
        &working_path,
        os_slash_str(),
        os_slash_str(),
        os_slash_str(),
        &safe_model_name
    );
    let model_suffix;
    if form.quantized {
        model_suffix = ".quant";
    } else {
        model_suffix = "";
    }
    let model_path = format!(
        "{}{}model{}.onnx",
        &model_dir_path,
        os_slash_str(),
        &model_suffix
    );
    let tokenizer_path = format!("{}{}tokenizer.json", &model_dir_path, os_slash_str(),);
    if !Path::new(&model_path).exists() {
        return not_ok_json_response(
            Status::BadRequest,
            make_bad_json_data_response(format!("Could not find model '{}'", &safe_model_name)),
        );
    }
   if !Path::new(&tokenizer_path).exists() {
        return not_ok_json_response(
            Status::BadRequest,
            make_bad_json_data_response(format!("Could not find tokenizer at '{}'", &tokenizer_path)),
        );
    }

    // Set up model
    let top_k = form.top_k;
    let temperature = form.temperature;
    let model = unsafe { Model::load_mmap(model_path) }.expect("load model");
    let tokenizer = Tokenizer::from_file(&tokenizer_path).expect("load tokenizer");
    let mut generator = generator_from_model(&model, &tokenizer, top_k, temperature);

    // Build reference for now
    let _bcv = format!("{} {} {} {} {}", &form.book, &form.from_chapter, &form.to_chapter.unwrap_or(form.from_chapter), &form.from_verse, &form.to_verse.unwrap_or(form.from_verse));

    // Query model
    let now = Instant::now();
    let output_tokens = do_one_iteration(
        &mut generator,
        &tokenizer,
        form.rag_context.clone(),
        form.prompt.clone(),
        form.show_prompt,
    ).expect("process prompt");

    // Make string from tokens
    let token_string: String = output_tokens.iter().map(|s| s.to_string()).collect();
    ok_json_response(json!({"isOk": true, "elapsed": now.elapsed().as_secs_f32(), "response": token_string}).to_string())  
}
