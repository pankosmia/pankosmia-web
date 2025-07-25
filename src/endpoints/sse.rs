use crate::static_vars::{DEBUG_IS_ENABLED, I18N_UPDATE_COUNT, NET_IS_ENABLED};
use crate::structs::AppSettings;
use crate::MsgQueue;
use rocket::response::stream;
use rocket::tokio::time;
use rocket::{get, State};
use std::sync::atomic::Ordering;
use std::time::Duration;

/// *`GET /`*
///
/// Typically mounted as **`/notifications/`**
///
/// Opens an SSE stream for notifications about server state. Use an existing SSE client to connect to this unless you like pain.
#[get("/")]
pub async fn notifications_stream<'a>(
    msgs: &'a State<MsgQueue>,
    state: &'a State<AppSettings>,
) -> stream::EventStream![stream::Event + 'a] {
    stream::EventStream! {
        let mut count = 0;
        let mut interval = time::interval(Duration::from_millis(500));
        yield stream::Event::retry(Duration::from_secs(1));
        let mut languages = state.languages.lock().unwrap().clone().join("/");
        let mut i18n_update_count = I18N_UPDATE_COUNT.load(Ordering::Relaxed);
        let mut typography = state.typography.lock().unwrap().clone();
        let mut bcv = state.bcv.lock().unwrap().clone();
        let gitea_endpoints = state.gitea_endpoints.clone();
        let mut auth_tokens = state.auth_tokens.lock().unwrap().clone();
        let mut current_project = state.current_project.lock().unwrap().clone();
        let mut first_time = true;
        loop {
            while !msgs.lock().unwrap().is_empty() {
                let msg = msgs.lock().unwrap().pop_front().unwrap();
                yield stream::Event::data(msg)
                    .event("misc")
                    .id(format!("{}", count));
                count+=1;
                interval.tick().await;
            };
            let net_status = match NET_IS_ENABLED.load(Ordering::Relaxed) {
                true => "enabled",
                false => "disabled"
            };
            let debug_status = match DEBUG_IS_ENABLED.load(Ordering::Relaxed) {
                true => "enabled",
                false => "disabled"
            };
            yield stream::Event::data(
                format!("net--{}--debug--{}", net_status, debug_status)
            )
            .event("status")
            .id(format!("{}", count));
            count+=1;
            let new_bcv = state.bcv.lock().unwrap().clone();
            if bcv.book_code != new_bcv.book_code || bcv.chapter != new_bcv.chapter || bcv.verse != new_bcv.verse || first_time  {
                bcv = new_bcv;
                yield stream::Event::data(
                    format!("{}--{}--{}", bcv.book_code, bcv.chapter, bcv.verse)
                )
                .event("bcv")
                .id(format!("{}", count));
                count+=1;
            }
            let new_current_project = state.current_project.lock().unwrap().clone();
            match  new_current_project {
                Some(np) => {
                    match current_project.clone() {
                        Some(op) => {
                            if np.source != op.source || np.organization != op.organization || np.project != op.project || first_time  {
                                current_project = Some(np.clone());
                                yield stream::Event::data(
                                    format!("{}--{}--{}", np.source, np.organization, np.project)
                                )
                                .event("current_project")
                                .id(format!("{}", count));
                                count+=1;
                        }
                    },
                        None => {
                                current_project = Some(np.clone());
                                yield stream::Event::data(
                                    format!("{}--{}--{}", np.source, np.organization, np.project)
                                )
                                .event("current_project")
                                .id(format!("{}", count));
                                count+=1;
                        }

                    }
                },
                None => {
                    match current_project.clone() {
                        None => {},
                        Some(_) => {
                            current_project = None;
                                yield stream::Event::data("null")
                                .event("current_project")
                                .id(format!("{}", count));
                                count+=1;
                        }
                    }
                }
            };
            let new_typography = state.typography.lock().unwrap().clone();
                if typography.font_set != new_typography.font_set || typography.size != new_typography.size || typography.direction != new_typography.direction || first_time  {
                typography = new_typography;
                yield stream::Event::data(
                    format!("{}--{}--{}", typography.font_set, typography.size, typography.direction)
                )
                .event("typography")
                .id(format!("{}", count));
                count+=1;
            }
            let new_auth_tokens = state.auth_tokens.lock().unwrap().clone();
            for (ep_name, ep_endpoint) in &gitea_endpoints {
                let ep_name2 = ep_name.clone();
                if new_auth_tokens.contains_key(&ep_name2) != auth_tokens.contains_key(&ep_name2) {
                    auth_tokens.insert(ep_name.clone(), new_auth_tokens[&ep_name2].clone());
                    yield stream::Event::data(
                        format!("{}--{}--{}", ep_name, ep_endpoint, auth_tokens.contains_key(&ep_name2))
                    )
                    .event("auth")
                    .id(format!("{}", count));
                    count+=1;
                } else if first_time {
                    yield stream::Event::data(
                        format!("{}--{}--{}", ep_name, ep_endpoint, auth_tokens.contains_key(&ep_name2))
                    )
                    .event("auth")
                    .id(format!("{}", count));
                    count+=1;
                }
            }
            let new_languages = state.languages.lock().unwrap().clone().join("/");
            let mut i18n_updated = false;
            let new_i18n_update = I18N_UPDATE_COUNT.load(Ordering::Relaxed);
            if new_i18n_update > i18n_update_count {
                i18n_updated = true;
                i18n_update_count = new_i18n_update;
            }
            if new_languages.clone() != languages.clone() || first_time || i18n_updated {
                languages = new_languages;
                yield stream::Event::data(
                    format!("{}", languages.clone())
                )
                .event("languages")
                .id(format!("{}", count));
                count+=1;
            }
            first_time = false;
            interval.tick().await;
        }
    }
}
