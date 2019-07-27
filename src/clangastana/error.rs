use std::fmt::{Display, Formatter};

#[derive(Fail, Debug)]
pub struct AstFileLoadError {
    ast_file: String,
}

impl AstFileLoadError {
    pub fn new(ast_file: String) -> Self {
        Self { ast_file }
    }
}

impl Display for AstFileLoadError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "ast file \"{}\" load error", &self.ast_file)
    }
}
