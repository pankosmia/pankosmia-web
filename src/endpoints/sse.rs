use std::sync::atomic::Ordering;
use std::time::Duration;
use rocket::tokio::time;
use rocket::{get, State};
use rocket::response::stream;
use crate::MsgQueue;
use crate::structs::AppSettings;
use crate::static_vars::{NET_IS_ENABLED, DEBUG_IS_ENABLED};

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
        let mut interval = time::interval(Duration::from_millis(1000));
        yield stream::Event::retry(Duration::from_secs(1));
        let mut languages = state.languages.lock().unwrap().clone().join("/");
        let mut typography = state.typography.lock().unwrap().clone();
        let mut bcv = state.bcv.lock().unwrap().clone();
        let gitea_endpoints = state.gitea_endpoints.clone();
        let mut auth_tokens = state.auth_tokens.lock().unwrap().clone();
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
            if bcv.book_code != new_bcv.book_code || bcv.chapter != new_bcv.chapter || bcv.verse != new_bcv.verse || count < 10  {
                bcv = new_bcv;
                yield stream::Event::data(
                    format!("{}--{}--{}", bcv.book_code, bcv.chapter, bcv.verse)
                )
                .event("bcv")
                .id(format!("{}", count));
                count+=1;
            }
            let new_typography = state.typography.lock().unwrap().clone();
                if typography.font_set != new_typography.font_set || typography.size != new_typography.size || typography.direction != new_typography.direction || count < 10  {
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
                } else if count < 10 {
                    yield stream::Event::data(
                        format!("{}--{}--{}", ep_name, ep_endpoint, auth_tokens.contains_key(&ep_name2))
                    )
                    .event("auth")
                    .id(format!("{}", count));
                    count+=1;
                }
            }
            let new_languages = state.languages.lock().unwrap().clone().join("/");
            if new_languages.clone() != languages.clone() || count < 10 {
                languages = new_languages;
                yield stream::Event::data(
                    format!("{}", languages.clone())
                )
                .event("languages")
                .id(format!("{}", count));
                count+=1;
            }
            interval.tick().await;
        }
    }
}
