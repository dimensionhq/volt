use cli_table::{format::Justify, Color, Table};
use serde::{Deserialize, Serialize};

// #[derive(Debug, Serialize, Deserialize, Default)]
// pub struct SearchResp{
//     vec: HashMap<String,Vec<Search>>
// }

#[derive(Debug, Serialize, Deserialize, Default, Table)]
pub struct SearchData {
    #[table(title = "Name", justify = "Justify::Right", color = "Color::Green")]
    name: String,
    #[table(title = "Version", color = "Color::Green")]
    version: String,
    #[table(title = "Description", color = "Color::Green")]
    description: String,
}
