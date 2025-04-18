use serde::{Deserialize, Serialize};
use std::sync::{Mutex};
use std::collections::{BTreeMap};
use std::fmt;
use rocket::fs::{TempFile};
use rocket::{Responder, FromForm};
use rocket::http::{ContentType};
use rocket::response::{status, Redirect};

#[derive(Debug)]
pub struct PankosmiaError(pub String);

impl fmt::Display for PankosmiaError {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PankosmiaError: {}", self.0)
    }
}

impl std::error::Error for PankosmiaError {}

#[derive(Serialize, Deserialize, Clone)]
pub struct Bcv {
    pub book_code: String,
    pub chapter: u16,
    pub verse: u16,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TypographyFeature {
    pub key: String,
    pub value: u8
}

impl fmt::Display for TypographyFeature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.key, self.value)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Typography {
    pub font_set: String,
    pub size: String,
    pub direction: String,
    pub features: BTreeMap<String, Vec<TypographyFeature>>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthRequest {
    pub code: String,
    pub redirect_uri: String,
    pub timestamp: std::time::SystemTime,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectIdentifier {
    pub source: String,
    pub organization: String,
    pub project: String
}

#[derive(Serialize, Deserialize)]
pub struct AppSettings {
    pub working_dir: String,
    pub repo_dir: Mutex<String>,
    pub app_resources_dir: String,
    pub languages: Mutex<Vec<String>>,
    pub gitea_endpoints: BTreeMap<String, String>,
    pub auth_tokens: Mutex<BTreeMap<String, String>>,
    pub auth_requests: Mutex<BTreeMap<String, AuthRequest>>,
    pub bcv: Mutex<Bcv>,
    pub typography: Mutex<Typography>,
    pub current_project: Mutex<Option<ProjectIdentifier>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum ClientSource {
    App,
    User
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Client {
    pub id: String,
    pub src: String,
    pub requires: BTreeMap<String, bool>,
    pub exclude_from_menu: bool,
    pub exclude_from_dashboard: bool,
    pub path: String,
    pub url: String,
}

#[derive(Serialize)]
pub struct PublicClient {
    pub id: String,
    pub requires: BTreeMap<String, bool>,
    pub exclude_from_menu: bool,
    pub exclude_from_dashboard: bool,
    pub url: String,
}

#[derive(Serialize, Deserialize)]
pub struct UserSettings {
    pub languages: Vec<String>,
    pub repo_dir: String,
    pub app_resources_dir: String,
    pub typography: Typography,
    pub my_clients: Vec<Client>,
    pub gitea_endpoints: BTreeMap<String, String>,
}

#[derive(Serialize, Deserialize)]
pub struct JsonDataResponse {
    pub is_good: bool,
    pub reason: String,
}
#[derive(Serialize, Deserialize)]
pub struct JsonNetStatusResponse {
    pub is_enabled: bool,
}

#[derive(Serialize, Deserialize)]
pub struct RemoteRepoRecord {
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
pub struct GitStatusRecord {
    pub path: String,
    pub change_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct MetadataSummary {
    pub name: String,
    pub description: String,
    pub flavor_type: String,
    pub flavor: String,
    pub language_code: String,
    pub script_direction: String,
}

#[derive(FromForm)]
pub struct Upload<'f> {
    pub file: TempFile<'f>,
}

#[derive(Responder)]
pub enum ContentOrRedirect {
    Content(status::Custom<(ContentType, String)>),
    Redirect(Redirect),
}