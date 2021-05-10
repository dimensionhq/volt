use json::JsonValue;

pub struct Package {
    pub name: String,
    pub url: JsonValue,
    pub developer: String,
}
