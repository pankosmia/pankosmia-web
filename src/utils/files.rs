use serde_json::Value;
use std::io::{Write};
use std::fs;
use std::path::Path;
use rocket::State;
use crate::utils::paths::{maybe_os_quoted_path_str, os_slash_str, user_settings_path};
use crate::structs::{PankosmiaError, AppSettings, UserSettings, TypographyFeature};
use crate::utils::client::Clients;

pub(crate) fn customize_and_copy_template_file(from_path: &str, to_path: &String, working_dir: &String, app_resources_dir: &String, pankosmia_dir: &String) -> Result<(), std::io::Error> {
    let json_string = fs::read_to_string(&from_path)?;
    let quoted_json_string = maybe_os_quoted_path_str(json_string
        .replace("%%WORKINGDIR%%", &working_dir)
        .replace("%%APPRESOURCESDIR%%", &app_resources_dir)
        .replace("%%PANKOSMIADIR%%", &pankosmia_dir)
    );
    let mut file_handle = fs::File::create(&to_path)?;
    file_handle.write_all(&quoted_json_string.as_bytes())?;
    Ok(())
}

pub(crate) fn load_json(from_path: &str) -> Result<Value, std::io::Error> {
    let json_string = fs::read_to_string(&from_path)?;
    Ok(serde_json::from_str(json_string.as_str())?)
}

pub(crate) fn load_and_substitute_json(from_path: &str, from_text: &str, to_text: &str) -> Result<Value, std::io::Error> {
    let mut json_string = fs::read_to_string(&from_path)?;
    json_string = json_string.replace(from_text, to_text);
    Ok(serde_json::from_str(json_string.as_str())?)
}

pub(crate) fn write_user_settings(state: &State<AppSettings>, clients: &State<Clients>) -> Result<(), std::io::Error> {
    let user_record = UserSettings {
        languages: (*state.languages.lock().unwrap()).to_owned(),
        repo_dir: state.repo_dir.lock().unwrap().clone(),
        app_resources_dir: state.app_resources_dir.clone(),
        typography: (*state.typography.lock().unwrap()).to_owned(),
        my_clients: clients.lock().unwrap().iter().filter(|c| { c.src == "User".to_string() }).map(|c| { c.clone() }).collect(),
        gitea_endpoints: state.gitea_endpoints.clone(),
    };
    let working_dir = state.working_dir.clone();
    let to_path = user_settings_path(&working_dir);
    let file_handle = fs::File::create(&to_path)?;
    serde_json::to_writer_pretty(file_handle, &user_record)?;
    Ok(())
}

pub(crate) fn copy_and_customize_webfont_css(template_path: &String, target_path: &String, user_settings: &Value, font_name: &String) -> Result<(), PankosmiaError> {
    let typography_json = user_settings["typography"].as_object().unwrap();
    let features_json = typography_json["features"].as_object().unwrap();
    let feature_json = match &features_json[font_name] {
        Value::Array(f) => f,
        _ => return Err(PankosmiaError(format!("No feature data in user_settings for font {}", font_name)))
    };
    let source_font_file_path = format!("{}{}pankosmia-{}.css", &template_path, os_slash_str(), &font_name);
    let target_font_file_path = format!("{}{}pankosmia-{}.css", &target_path, os_slash_str(), &font_name);
    if Path::new(&source_font_file_path).is_file() {
        let mut css_string = match fs::read_to_string(&source_font_file_path) {
            Ok(css_string) => css_string,
            Err(e) => {
                panic!("Could not read css file '{}': {}", source_font_file_path, e);
            }
        };
        let mut font_feature_collector = Vec::new();
        for feature_pair in feature_json {
            font_feature_collector.push(format!("{}: {}", feature_pair["key"], feature_pair["value"]));
        }
        css_string = css_string.replace("%%FONTFEATURES%%", &font_feature_collector.join(", "));
        match fs::write(&target_font_file_path, css_string) {
            Ok(_) => Ok(()),
            Err(e) => {
                panic!("Could not write css file '{}': {}", target_font_file_path, e);
            }
        }
    } else {
        Err(PankosmiaError(format!("Could not find source font file '{}'", source_font_file_path)))
    }
}

pub(crate) fn copy_and_customize_webfont_css2(template_path: &String, target_path: &String, font_features: &Vec<TypographyFeature>, font_name: &String) -> Result<(), PankosmiaError> {
    let source_font_file_path = format!("{}{}pankosmia-{}.css", &template_path, os_slash_str(), &font_name);
    let target_font_file_path = format!("{}{}pankosmia-{}.css", &target_path, os_slash_str(), &font_name);
    if Path::new(&source_font_file_path).is_file() {
        let mut css_string = match fs::read_to_string(&source_font_file_path) {
            Ok(css_string) => css_string,
            Err(e) => {
                panic!("Could not read css file '{}': {}", source_font_file_path, e);
            }
        };
        let mut font_feature_collector = Vec::new();
        for feature_pair in font_features {
            font_feature_collector.push(format!("\"{}\" {}", feature_pair.key, feature_pair.value));
        }
        css_string = css_string.replace("%%FONTFEATURES%%", &font_feature_collector.join(", "));
        match fs::write(&target_font_file_path, css_string) {
            Ok(_) => Ok(()),
            Err(e) => {
                panic!("Could not write css file '{}': {}", target_font_file_path, e);
            }
        }
    } else {
        Err(PankosmiaError(format!("Could not find source font file '{}'", source_font_file_path)))
    }
}