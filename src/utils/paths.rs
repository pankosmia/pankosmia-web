use std::env;
use std::path::{Components};
use home::home_dir;

pub(crate) fn os_slash_str() -> &'static str {
    match env::consts::OS {
        "windows" => "\\",
        _ => "/",
    }
}

pub(crate) fn maybe_os_quoted_path_str(s: String) -> String {
    match env::consts::OS {
        "windows" => s.replace("\\", "\\\\").replace("/", "\\\\"),
        _ => s,
    }
}

pub(crate) fn forbidden_path_strings() -> Vec<String> {
    Vec::from([
        "..".to_string(),
        "~".to_string(),
        "/".to_string(),
        "\\".to_string(),
        "&".to_string(),
        "*".to_string(),
        "+".to_string(),
        "|".to_string(),
        " ".to_string(),
        "?".to_string(),
        "#".to_string(),
        "%".to_string(),
        "{".to_string(),
        "}".to_string(),
        "<".to_string(),
        ">".to_string(),
        "$".to_string(),
        "!".to_string(),
        "'".to_string(),
        "\"".to_string(),
        ":".to_string(),
        ";".to_string(),
        "`".to_string(),
        "=".to_string(),
    ])
}

pub(crate) fn check_path_components(path_components: &mut Components<'_>) -> bool {
    let mut ret = true;
    if path_components.clone().collect::<Vec<_>>().len() < 1 {
        return false;
    }
    for path_component in path_components {
        let path_string = path_component
            .clone()
            .as_os_str()
            .to_str()
            .unwrap()
            .to_string();
        if path_string.starts_with(".") {
            return false;
        }
        for forbidden_string in forbidden_path_strings() {
            if path_string.contains(&forbidden_string) {
                ret = false;
                break;
            }
        }
    }
    ret
}
pub(crate) fn check_path_string_components(path_string: String) -> bool {
    let mut ret = true;
    if path_string.starts_with("/") {
        return false;
    }
    let path_string_parts = path_string.split("/");
    for path_string_part in path_string_parts {
        if path_string_part.len() < 1 {
            return false;
        }
        if path_string_part.starts_with(".") {
            return false;
        }
        for forbidden_string in forbidden_path_strings() {
            if path_string_part.contains(&forbidden_string) {
                ret = false;
                break;
            }
        }
    }
    ret
}

pub(crate) fn home_dir_string() -> String {
    home_dir()
        .unwrap()
        .as_os_str()
        .to_str()
        .unwrap()
        .to_string()
}

pub(crate) fn app_setup_path (working_dir: &String) -> String {
    format!("{}/app_setup.json", working_dir)
}

pub(crate) fn app_state_path (working_dir: &String) -> String {
    format!("{}/app_state.json", working_dir)
}

pub(crate) fn user_settings_path (working_dir: &String) -> String {
    format!("{}/user_settings.json", working_dir)
}

pub(crate) fn webfonts_path (working_dir: &String) -> String {
    format!("{}/webfonts", working_dir)
}

pub(crate) fn source_webfonts_path(app_resources_dir: &String) -> String {
    format!("{}/webfonts", app_resources_dir)
}

pub(crate) fn source_app_resources_path(app_resources_dir: &String) -> String {
    format!("{}/app_resources/", app_resources_dir)
}