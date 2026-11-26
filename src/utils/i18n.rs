use crate::structs::AppSettings;

pub(crate) fn do_i18n(app_settings: &AppSettings, category: &str, key: &str, substitutions: Vec<String>) -> (bool, String, String) {
    let mut res = (false, key.to_string(), key.to_string());
    let i18n = &app_settings.i18n;
    let full_key = format!("{}--{}", &category, &key);
    if i18n.contains_key(&full_key) {
            let key_i18n = &i18n[&full_key].clone();
            let languages = app_settings.languages.lock().unwrap().clone();
            for language in languages {
                if key_i18n.contains_key(&language) {
                    let mut localized = key_i18n[&language].clone();
                    for substitution in substitutions {
                        localized = localized.replacen("{}", substitution.as_str(), 1)
                    }
                    res = (true, key.to_string(), localized.to_string());
                    break;
                }
            }
        }
    res
}