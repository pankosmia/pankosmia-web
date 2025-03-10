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
        let mut interval = time::interval(Duration::from_millis(500));
        yield stream::Event::retry(Duration::from_secs(1));
        loop {
            while !msgs.lock().unwrap().is_empty() {
                let msg = msgs.lock().unwrap().pop_front().unwrap();
                yield stream::Event::data(msg)
                    .event("misc")
                    .id(format!("{}", count));
                count+=1;
                interval.tick().await;
            };
            yield stream::Event::data(
                    match NET_IS_ENABLED.load(Ordering::Relaxed) {
                        true => "enabled",
                        false => "disabled"
                    }
            )
            .event("net_status")
            .id(format!("{}", count));
            count+=1;
            yield stream::Event::data(
                    match DEBUG_IS_ENABLED.load(Ordering::Relaxed) {
                        true => "enabled",
                        false => "disabled"
                    }
            )
            .event("debug")
            .id(format!("{}", count));
            count+=1;
            if count % 4 == 0 {
                let bcv = state.bcv.lock().unwrap().clone();
                yield stream::Event::data(
                    format!("{}--{}--{}", bcv.book_code, bcv.chapter, bcv.verse)
                )
                .event("bcv")
                .id(format!("{}", count));
                count+=1;
                /*
            } else if count % 4 == 1 {
                let typography = state.typography.lock().unwrap().clone();
                yield stream::Event::data(
                    format!("{}--{}--{}", typography.font_set, typography.size, typography.direction)
                )
                .event("typography")
                .id(format!("{}", count));
                count+=1;
                 */
            } else if count % 4 == 2 {
                let gitea_endpoints = state.gitea_endpoints.clone();
                let auth_tokens = state.auth_tokens.lock().unwrap().clone();
                for (ep_name, ep_endpoint) in gitea_endpoints {
                    yield stream::Event::data(
                        format!("{}--{}--{}", ep_name, ep_endpoint, auth_tokens.contains_key(&ep_name))
                    )
                    .event("auth")
                    .id(format!("{}", count));
                    count+=1;
                }
                /*
            } else if count % 4 == 3 {
                let languages = state.languages.lock().unwrap().clone();
                yield stream::Event::data(
                    format!("{}", languages.join("/"))
                )
                .event("languages")
                .id(format!("{}", count));
                count+=1;
                 */
            }
            interval.tick().await;
        }
    }
}
