use crate::structs::{AppSettings, Typography, TypographyFeature};
use crate::utils::client::Clients;
use crate::utils::files::{copy_and_customize_webfont_css2, write_user_settings};
use crate::utils::json_responses::{make_bad_json_data_response};
use crate::utils::paths::{source_webfonts_path, webfonts_path};
use crate::utils::response::{not_ok_json_response, ok_ok_json_response};
use crate::MsgQueue;
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{post, State};
use std::collections::BTreeMap;

/// *`POST /typography-feature/<font_name>/<feature>/<value>`*
///
/// Typically mounted as **`/settings//typography-feature/<font_name>/<feature>/<value>`**
///
/// Sets the value of a font feature. Currently silently ignores unknown fonts and fields.
#[allow(irrefutable_let_patterns)]
#[post("/typography-feature/<font_name>/<feature>/<new_value>")]
pub fn post_typography_feature(
    state: &State<AppSettings>,
    clients: &State<Clients>,
    msgs: &State<MsgQueue>,
    font_name: &str,
    feature: &str,
    new_value: u8,
) -> status::Custom<(ContentType, String)> {
    if let mut typo_inner = state.typography.lock().unwrap() {
        let mut new_font_fields = BTreeMap::new();
        for (font_key, font_value) in &mut typo_inner.features {
            if font_key == font_name {
                let mut new_fields = Vec::new();
                let font_inner = &mut *font_value;
                for field_kv in font_inner.clone() {
                    if field_kv.key == feature {
                        new_fields.push(TypographyFeature {
                            key: field_kv.key.to_string(),
                            value: new_value,
                        });
                    } else {
                        new_fields.push(TypographyFeature {
                            key: field_kv.key.to_string(),
                            value: field_kv.value,
                        });
                    }
                }
                let working_dir = state.working_dir.clone();
                let app_resources_dir = state.app_resources_dir.clone();
                let src_webfonts_dir = source_webfonts_path(&app_resources_dir);
                let target_webfonts_dir = webfonts_path(&working_dir);
                match copy_and_customize_webfont_css2(
                    &src_webfonts_dir,
                    &target_webfonts_dir,
                    &new_fields,
                    &font_name.to_string(),
                ) {
                    Ok(_) => {}
                    Err(e) => {
                        return not_ok_json_response(
                            Status::BadRequest,
                            make_bad_json_data_response(format!("Could not rewrite CSS: {}", e)),
                        );
                    }
                }
                new_font_fields.insert(font_key.to_string(), new_fields);
            } else {
                new_font_fields.insert(font_key.to_string(), font_value.to_vec());
            }
        }
        *typo_inner = Typography {
            font_set: typo_inner.font_set.clone(),
            size: typo_inner.size.clone(),
            direction: typo_inner.direction.clone(),
            features: new_font_fields,
        };
        msgs.lock()
            .unwrap()
            .push_back("info--3--typography-feature--change".to_string());
    }
    match write_user_settings(&state, &clients) {
        Ok(_) => {}
        Err(e) => {
            return not_ok_json_response(
                Status::InternalServerError,
                make_bad_json_data_response(format!("Could not write out user settings: {}", e)),
            )
        }
    }
    ok_ok_json_response()
}
