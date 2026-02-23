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
use serde::{Deserialize, Serialize};
use std::time::{Instant, SystemTime};

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
    show_prompt: bool,
}

#[derive(Serialize)]
pub struct RagResponse {
    submitted: f32,
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
    is_ok: bool,
    elapsed: Option<f32>,
    response: Option<String>,
}

/// *`POST /rag-prompt`*
///
/// Typically mounted as **`/llm/rag-prompt`**
///
/// Builds a RAG prompt, processes it and returns the result
/// 
/// Request JSON
/// 
/// {
///  "model_name": "qwen3-4b",
///  "quantized": true,
///  "book": "JHN",
///  "from_chapter": 3,
///  "from_verse": 16,
///  "prompt": "Who loves the world?",
///  "show_prompt": false,
///  "top_k": 20,
///  "temperature": 0.5,
///   "rag_context": {
///     "juxta": "This verse is one complete Greek sentence...",
///     "translations": {
///         "Berean Standard Bible": "For God so loved the world...",
///         "unfoldingWord Simplified Text": "This is because God loved the world’s people in this way..."},
///     "notes": {
///       "Tyndale Study Notes": ["The truth that God loved the world is basic to..."]
///     },
///     "snippets":{
///       "for": [
///         "For here indicates that..."
///       ],
///       "one and only son": [
///         "Here, One and Only Son refers to...",
///         "Here and throughout John’s Gospel, the phrase One and Only is..."
///       ]
///     }
///   },
/// 
/// *Response JSON*
/// {
///   "submitted": 1771843000.0,
///   "model_name": "qwen3-4b",
///   "quantized": true,
///   "book": "JHN",
///   "from_chapter": 3,
///   "to_chapter": null,
///   "from_verse": 16,
///   "to_verse": null,
///   "rag_context": {
///     "juxta": "This verse is one complete Greek sentence...",
///     "translations": {
///         "Berean Standard Bible": "For God so loved the world...",
///         "unfoldingWord Simplified Text": "This is because God loved the world’s people in this way..."},
///     "notes": {
///       "Tyndale Study Notes": ["The truth that God loved the world is basic to..."]
///     },
///     "snippets":{
///       "for": [
///         "For here indicates that..."
///       ],
///       "one and only son": [
///         "Here, One and Only Son refers to...",
///         "Here and throughout John’s Gospel, the phrase One and Only is..."
///       ]
///     }
///   },
///   "top_k": 20,
///   "temperature": 0.5,
///   "prompt": "Who loves the world?",
///   "is_ok": true,
///   "elapsed": 61.559628,
///   "response": "</think>\n</think>\n\nGod loves the world. The verse says..."
/// }
#[post("/rag-prompt", format = "json", data = "<form>")]
pub async fn post_rag_prompt(
    state: &State<AppSettings>,
    form: Json<RagPromptForm>,
) -> status::Custom<(ContentType, String)> {
    let safe_field_regex = Regex::new(r"[^A-Za-z0-9._-]").unwrap();
    let safe_model_name = safe_field_regex.replace(form.model_name.as_str(), "_");
    let working_path = state.working_dir.clone();
        let submitted = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("epoch time")
            .as_secs_f32();

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
            make_bad_json_data_response(format!(
                "Could not find tokenizer at '{}'",
                &tokenizer_path
            )),
        );
    }

    // Set up model
    let top_k = form.top_k;
    let temperature = form.temperature;
    let model = unsafe {
        match Model::load_mmap(model_path) {
            Ok(m) => m,
            Err(e) => {
                return not_ok_json_response(
                    Status::BadRequest,
                    make_bad_json_data_response(format!("Could not load model: '{}'", &e)),
                )
            }
        }
    };
    let tokenizer = match Tokenizer::from_file(&tokenizer_path) {
        Ok(t) => t,
        Err(e) => {
            return not_ok_json_response(
                Status::BadRequest,
                make_bad_json_data_response(format!("Could not load tokenizer: '{}'", &e)),
            )
        }
    };
    let mut generator = generator_from_model(&model, &tokenizer, top_k, temperature);

    // Build reference for now
    let _bcv = format!(
        "{} {} {} {} {}",
        &form.book,
        &form.from_chapter,
        &form.to_chapter.unwrap_or(form.from_chapter),
        &form.from_verse,
        &form.to_verse.unwrap_or(form.from_verse)
    );

    // Query model
    let now = Instant::now();
    let output_tokens = match do_one_iteration(
        &mut generator,
        &tokenizer,
        form.rag_context.clone(),
        form.prompt.clone(),
        form.show_prompt,
    ) {
        Ok(t) => t,
        Err(e) => {
            return not_ok_json_response(
                Status::BadRequest,
                make_bad_json_data_response(format!("Prompt processing failed: '{}'", &e)),
            )
        }
    };

    // Make response struct and return
    let token_string: String = output_tokens.iter().map(|s| s.to_string()).collect();
    let response = RagResponse {
        submitted: submitted,
        model_name: form.model_name.clone(),
        quantized: form.quantized,
        book: form.book.clone(),
        from_chapter: form.from_chapter,
        to_chapter: form.to_chapter,
        from_verse: form.from_verse,
        to_verse: form.to_verse,
        rag_context: form.rag_context.clone(),
        top_k: form.top_k,
        temperature: form.temperature,
        prompt: form.prompt.clone(),
        is_ok: true,
        elapsed: Some(now.elapsed().as_secs_f32()),
        response: Some(token_string),
    };

    ok_json_response(serde_json::to_string_pretty(&response).expect("serialize_response"))
}
