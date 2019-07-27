use clang::SourceError;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io::Error as IoError;
use xml::writer::Error as XmlError;

#[derive(Debug)]
pub enum AstXmlError {
    Clang(SourceError),
    AstLoad(String),
    Xml(XmlError),
    Io(IoError),
}

impl Display for AstXmlError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            AstXmlError::Clang(e) => write!(f, "${:?}", e),
            AstXmlError::AstLoad(filepath) => write!(f, "ast file \"{}\" load error", &filepath),
            AstXmlError::Xml(e) => write!(f, "${:?}", e),
            AstXmlError::Io(e) => write!(f, "${:?}", e),
        }
    }
}

impl From<XmlError> for AstXmlError {
    fn from(e: XmlError) -> Self {
        AstXmlError::Xml(e)
    }
}

impl From<SourceError> for AstXmlError {
    fn from(e: SourceError) -> Self {
        AstXmlError::Clang(e)
    }
}

impl From<IoError> for AstXmlError {
    fn from(e: IoError) -> Self {
        AstXmlError::Io(e)
    }
}

impl Error for AstXmlError {}
