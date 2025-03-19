use serde_json::Value;
use std::io::{Write};
use crate::utils::paths::maybe_os_quoted_path_str;

pub(crate) fn customize_and_copy_template_file(from_path: &str, to_path: &String, working_dir: &String) -> Result<(), std::io::Error> {
    let json_string = std::fs::read_to_string(&from_path)?;
    let quoted_json_string = maybe_os_quoted_path_str(json_string.replace("%%WORKINGDIR%%", &working_dir));
    let mut file_handle = std::fs::File::create(&to_path)?;
    file_handle.write_all(&quoted_json_string.as_bytes())?;
    Ok(())
}

pub(crate) fn load_json(from_path: &str) -> Result<Value, std::io::Error> {
    let json_string = std::fs::read_to_string(&from_path)?;
    Ok(serde_json::from_str(json_string.as_str())?)
}

pub(crate) fn load_and_substitute_json(from_path: &str, from_text: &str, to_text: &str) -> Result<Value, std::io::Error> {
    let mut json_string = std::fs::read_to_string(&from_path)?;
    json_string = json_string.replace(from_text, to_text);
    Ok(serde_json::from_str(json_string.as_str())?)
}
