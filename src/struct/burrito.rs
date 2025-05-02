use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BurritoMetadataIngredient {
    pub checksum: Value,
    pub mimeType: String,
    pub scope: Value,
    pub size: u32
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BurritoMetadata {
    pub format : String,
    pub meta: Value::Object,
    pub idAuthorities: Value::Object,
    pub identification: Value::Object,
    pub languages: Vec<Value::Object>,
    pub r#type: Value::Object,
    pub confidential: bool,
    pub localizedNames: Value::Object,
    pub ingredients: BTreeMap<String, BurritoMetadataIngredient>,
    pub copyright: Value::Object,
}
