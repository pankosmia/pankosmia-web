use serde::{Deserialize, Serialize};
use std::sync::{Mutex};
use std::collections::{BTreeMap};
use rocket::fs::{TempFile};
use rocket::{Responder, FromForm};
use rocket::http::{ContentType};
use rocket::response::{status, Redirect};

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Bcv {
    pub book_code: String,
    pub chapter: u16,
    pub verse: u16,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Typography {
    pub font_set: String,
    pub size: String,
    pub direction: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct AuthRequest {
    pub code: String,
    pub redirect_uri: String,
    pub timestamp: std::time::SystemTime,
}
#[derive(Serialize, Deserialize)]
pub(crate) struct AppSettings {
    pub working_dir: String,
    pub repo_dir: Mutex<String>,
    pub languages: Mutex<Vec<String>>,
    pub gitea_endpoints: BTreeMap<String, String>,
    pub auth_tokens: Mutex<BTreeMap<String, String>>,
    pub auth_requests: Mutex<BTreeMap<String, AuthRequest>>,
    pub bcv: Mutex<Bcv>,
    pub typography: Mutex<Typography>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct JsonDataResponse {
    pub is_good: bool,
    pub reason: String,
}
#[derive(Serialize, Deserialize)]
pub(crate) struct JsonNetStatusResponse {
    pub is_enabled: bool,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct RemoteRepoRecord {
    pub name: String,
    pub abbreviation: String,
    pub description: String,
    pub avatar_url: String,
    pub flavor: String,
    pub flavor_type: String,
    pub language_code: String,
    pub script_direction: String,
    pub branch_or_tag: String,
    pub clone_url: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct GitStatusRecord {
    pub path: String,
    pub change_type: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct MetadataSummary {
    pub name: String,
    pub description: String,
    pub flavor_type: String,
    pub flavor: String,
    pub language_code: String,
    pub script_direction: String,
}

#[derive(FromForm)]
pub(crate) struct Upload<'f> {
    pub file: TempFile<'f>,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Client {
    pub id: String,
    pub requires: BTreeMap<String, bool>,
    pub exclude_from_menu: bool,
    pub exclude_from_dashboard: bool,
    pub path: String,
    pub url: String,
}

#[derive(Serialize)]
pub(crate) struct PublicClient {
    pub id: String,
    pub requires: BTreeMap<String, bool>,
    pub exclude_from_menu: bool,
    pub exclude_from_dashboard: bool,
    pub url: String,
}

#[derive(Responder)]
pub(crate) enum ContentOrRedirect {
    Content(status::Custom<(ContentType, String)>),
    Redirect(Redirect),
}