use std::path::Path;

#[allow(clippy::module_name_repetitions)]
pub trait PathExtensions {
    fn file_name_as_str(&self) -> Option<&str>;
    fn file_name_as_string(&self) -> Option<String>;
}

impl PathExtensions for Path {
    fn file_name_as_str(&self) -> Option<&str> {
        self.file_name()?.to_str()
    }

    fn file_name_as_string(&self) -> Option<String> {
        self.file_name_as_str().map(String::from)
    }
}
