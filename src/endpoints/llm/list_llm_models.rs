use std::path::Path;
use crate::structs::AppSettings;
use crate::utils::response::ok_json_response;
use rocket::http::{ContentType};
use rocket::response::status;
use rocket::{get, State};
use crate::utils::paths::os_slash_str;

/// *`GET /model`*
///
/// Typically mounted as **`/llm/model`**
///
/// Returns a JSON array of llm models in the blobs/llm_models directory.
///
/// `["qwen3-4b/model.quant.onnx"]`
#[get("/model")]
pub fn list_llm_models(state: &State<AppSettings>) -> status::Custom<(ContentType, String)> {
    let working_path = state.working_dir.clone();
    let llm_models_path = format!(
        "{}{}blobs{}llm_models",
        &working_path,
        os_slash_str(),
        os_slash_str()
    );
    let llm_paths = match std::fs::read_dir(llm_models_path) {
        Ok(sp) => sp,
        Err(_) => return ok_json_response("[]".to_string())

    };
    let mut repos = Vec::new();
    for llm_path_result in llm_paths {
        let llm_path = llm_path_result.expect("llm_path");
        let llm_path_ob = llm_path.path();
        let llm_path_string = llm_path_ob.to_str().unwrap();
        let llm_leaf = llm_path_ob.file_name().unwrap();
        let llm_leaf_string = llm_leaf.to_str().unwrap();
        if llm_leaf.to_str().unwrap().starts_with(".") {
            println!("Skipping . file or dir {}", &llm_leaf.to_str().unwrap());
            continue;
        }
        if !Path::new(&llm_path_ob).is_dir() {
            println!("Skipping non-dir {}", llm_leaf.to_string_lossy());
            continue;
        }
        let non_quantized_path = format!(
            "{}{}model.onnx",
            &llm_path_string,
            os_slash_str()
        );
        println!("{}", &non_quantized_path);
        if Path::new(&non_quantized_path).exists() {
            repos.push((format!("{}", &llm_leaf_string), false))
        }
        let quantized_path = format!(
            "{}{}model.quant.onnx",
            &llm_path_string,
            os_slash_str()
        );
        if Path::new(&quantized_path).exists() {
            repos.push((format!("{}", &llm_leaf_string), true))
        }
    }
    ok_json_response(serde_json::to_string(&repos).unwrap())
}
