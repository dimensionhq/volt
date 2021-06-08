#[derive(Debug)]
pub struct JsonMinifier {
    pub is_string: bool,
    pub escaped_quotation: u8,
}

impl Default for JsonMinifier {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonMinifier {
    pub fn new() -> Self {
        JsonMinifier {
            is_string: false,
            escaped_quotation: 0,
        }
    }
}

#[inline]
pub fn keep_element(minifier: &mut JsonMinifier, item1: &char, item2: Option<&char>) -> bool {
    let remove_element =
        item1.is_ascii_control() || is_whitespace_outside_string(minifier, item1, item2);
    !remove_element
}

#[inline]
fn is_whitespace_outside_string(
    minifier: &mut JsonMinifier,
    item1: &char,
    item2: Option<&char>,
) -> bool {
    if !minifier.is_string && item1.eq(&'"') {
        minifier.is_string = true;
    } else if minifier.is_string {
        if item1.eq(&'\\') && item2.eq(&Some(&'"')) {
            minifier.escaped_quotation = 4;
        }
        if minifier.escaped_quotation > 0 {
            minifier.escaped_quotation -= 1;
        } else if item1.eq(&'"') {
            minifier.is_string = false;
        }
    }
    !minifier.is_string && item1.is_whitespace()
}
