use serde_json::Value;

pub fn validate_json_value(schema_spec: Value, json_value: &Value) -> Vec<String> {
        match jsonschema::validate(&schema_spec, json_value) {
            Ok(()) => vec![],
            Err(e) => vec![e.to_string()],
    }
}