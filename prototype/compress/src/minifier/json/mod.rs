#![allow(dead_code)]

use std::io::Read;
use {
    json_minifier::{keep_element, JsonMinifier},
    read::json_read::JsonRead,
    string::JsonMultiFilter,
};

mod read {
    mod byte_to_char;
    mod internal_buffer;
    mod internal_reader;
    pub mod json_read;
}

pub mod json_minifier;
pub mod string;

type JsonMethod = fn(&mut JsonMinifier, &char, Option<&char>) -> bool;

/// Minifies a given String by JSON minification rules
///
/// # Example
///
/// ```rust
/// extern crate minifier;
/// use minifier::json::minify;
///
/// fn main() {
///     let json = r#"
///            {
///                "test": "test",
///                "test2": 2
///            }
///        "#.into();
///     let json_minified = minify(json);
/// }
/// ```
#[inline]
pub fn minify(json: &str) -> String {
    JsonMultiFilter::new(json.chars(), keep_element).collect()
}

/// Minifies a given Read by JSON minification rules
///
/// # Example
///
/// ```rust
/// extern crate minifier;
/// use std::fs::File;
/// use std::io::Read;
/// use minifier::json::minify_from_read;
///
/// fn main() {
///     let mut html_minified = String::new();
///     let mut file = File::open("tests/files/test.json").expect("file not found");
///     minify_from_read(file).read_to_string(&mut html_minified);
/// }
/// ```
#[inline]
pub fn minify_from_read<R: Read>(json: R) -> JsonRead<JsonMethod, R> {
    JsonRead::new(json, keep_element)
}

#[test]
fn removal_from_read() {
    use std::fs::File;

    let input = File::open("tests/files/test.json").expect("file not found");
    let expected: String = "{\"test\":\"\\\" test2\",\"test2\":\"\",\"test3\":\" \"}".into();
    let mut actual = String::new();
    minify_from_read(input)
        .read_to_string(&mut actual)
        .expect("error at read");
    assert_eq!(actual, expected);
}

#[test]
fn removal_of_control_characters() {
    let input = "\n".into();
    let expected: String = "".into();
    let actual = minify(input);
    assert_eq!(actual, expected);
}

#[test]
fn removal_of_whitespace_outside_of_tags() {
    let input = r#"
            {
              "test": "\" test2",
              "test2": "",
              "test3": " "
            }
        "#
    .into();
    let expected: String = "{\"test\":\"\\\" test2\",\"test2\":\"\",\"test3\":\" \"}".into();
    let actual = minify(input);
    assert_eq!(actual, expected);
}
