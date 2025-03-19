use serde_json::Value;

pub(crate) fn get_string_value_by_key<'a>(value: &'a Value, key: &'a str) -> &'a String {
    match &value[key] {
        Value::String(v) => v,
        _ => {
            panic!(
                "Could not get string value for key  '{}'",
                key
            );
        }
    }
}
