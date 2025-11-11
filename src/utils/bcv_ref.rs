use serde_json::Value;
use crate::utils::paths::os_slash_str;

pub fn canonical_book_codes(app_resources_dir: String) -> Vec<String> {
    let versification_path = format!(
        "{}{}{}{}{}{}{}{}eng.json",
        app_resources_dir,
        os_slash_str(),
        "templates",
        os_slash_str(),
        "content_templates",
        os_slash_str(),
        "vrs",
        os_slash_str(),
    );

    let mut books = Vec::new();
    match std::fs::read_to_string(versification_path) {
        Ok(vrs_string) => {
            let vrs_json: Value = serde_json::from_str(vrs_string.as_str()).expect("VRS as JSON");
            let max_verses = vrs_json["maxVerses"].as_object().expect("maxVerses");
            for (key, _value) in max_verses {
                books.push(key.clone());
            }
        },
        Err(_) => panic!("Read VRS")
    }
    books
}