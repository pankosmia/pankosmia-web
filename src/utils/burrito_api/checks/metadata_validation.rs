use boon::{Compiler, Schemas};
use crate::utils::burrito_api::checks::report_helpers::{ok_check_report, CheckReport};

// Run basic_shape checks first
pub(crate) fn check_metadata_validation(burrito_path: String) -> Vec<CheckReport> {
    let mut reports = vec![];
    let schema_path = std::path::absolute(std::path::Path::new(
        "lib/app_resources/schema/scripture_burrito_metadata_schema/source_metadata.schema.json",
    ))
    .unwrap();
    let schema_path_str = schema_path.to_str().unwrap();
    let mut schemas = Schemas::new();
    let mut compiler = Compiler::new();
    let sch_index = compiler.compile(schema_path_str, &mut schemas).expect("Cannot comple schema");
    let metadata_path = format!("{}/metadata.json", burrito_path);
    let metadata_string = std::fs::read_to_string(&metadata_path)
        .expect(format!("Unable to read metadata from {}", metadata_path).as_str());
    let metadata_json = serde_json::from_str(&metadata_string).unwrap();
    match schemas.validate(&metadata_json, sch_index) {
        Ok(_) => reports.push(ok_check_report(
            "Metadata:Validation".to_string(),
            burrito_path.clone(),
        )),
        Err(errors) => {
            reports.push(
                CheckReport {
                    name: "Metadata:Validation:Validates".to_string(),
                    path: burrito_path.clone(),
                    success: false,
                    comment: Some("Metadata is not schema valid".to_string()),
                    data: Some(vec!(format!("{}", errors.detailed_output()))),
                }
            );
        }
    }

    reports
}
