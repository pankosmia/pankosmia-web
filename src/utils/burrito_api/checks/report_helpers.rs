use rocket::serde::Serialize;

#[derive(Serialize, Clone)]
pub(crate) struct CheckReport {
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) success: bool,
    pub(crate) comment: Option<String>,
    pub(crate) data: Option<Vec<String>>,
}

pub(crate) fn ok_check_report(name: String, path: String) -> CheckReport {
    CheckReport {
        name,
        path,
        success: true,
        comment: None,
        data: None,
    }
}
