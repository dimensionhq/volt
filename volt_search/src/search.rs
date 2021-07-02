use serde::{Deserialize, Serialize};

// #[derive(Debug, Serialize, Deserialize, Default)]
// pub struct SearchResp{
//     vec: HashMap<String,Vec<Search>>
// }

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SearchData{
    pub name: String,
    pub version: String,
    pub description: String
}