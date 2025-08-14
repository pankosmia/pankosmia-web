use valico::json_schema;
use crate::utils::burrito_api::checks::report_helpers::{ok_check_report, CheckReport};

// Run basic_shape checks first
pub(crate) fn check_metadata_validation(burrito_path: String) -> Vec<CheckReport> {
    let mut reports = vec![];
    let schema_path = std::path::absolute(std::path::Path::new(
        "lib/app_resources/schema/usj_schema.json",
    ))
    .unwrap();
    let schema_path_str = schema_path.to_str().unwrap();
    let schema_string = std::fs::read_to_string(schema_path_str).unwrap();
    let schema_json = serde_json::from_str(&schema_string).unwrap();
    let mut scope = json_schema::Scope::new();
    let validator = scope.compile_and_return(schema_json, false).expect("Cannot make JSON validator");
    let metadata_path = format!("{}/metadata.json", burrito_path);
    let metadata_string = std::fs::read_to_string(metadata_path).unwrap();
    let metadata_json = serde_json::from_str(&metadata_string).unwrap();
    let validation = validator.validate(&metadata_json);
    let validation_errors = validation.errors.into_iter().map(|e| e.to_string()).collect::<Vec<String>>();
    println!("Validation errors: {:?}", validator.validate(&metadata_json).is_strictly_valid());
    if validation_errors.is_empty() {
        reports.push(ok_check_report(
            "Metadata:Validation".to_string(),
            burrito_path.clone(),
        ))
    } else {
        reports.push(
            CheckReport {
                name: "Metadata:Validation:Validates".to_string(),
                path: burrito_path.clone(),
                success: false,
                comment: Some("Metadata is not schema valid".to_string()),
                data: Some(validation_errors),
            }
        );
    }
    reports
}
