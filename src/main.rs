use core::result::Result;
use std::env;

use clang::SourceError;
use clang::{Clang, Entity, EntityVisitResult, Index, TranslationUnit};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io::Write;
use xml::writer::Error as XmlError;
use xml::writer::XmlEvent;
use xml::{EmitterConfig, EventWriter};

#[derive(Debug)]
pub enum AstXmlError {
    Clang(SourceError),
    Xml(XmlError),
}

impl Display for AstXmlError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            AstXmlError::Clang(e) => write!(f, "${:?}", e),
            AstXmlError::Xml(e) => write!(f, "${:?}", e),
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

impl Error for AstXmlError {}

pub fn create_start_xml_event_from_entry<W: Write>(
    entry: Entity,
    writer: &mut EventWriter<W>,
) -> Result<(), XmlError> {
    let kind = format!("{:?}", entry.get_kind());
    let elem = XmlEvent::start_element(kind.as_str());
    writer.write(XmlEvent::from(elem))
}

pub fn create_end_xml_event_from_entry<W: Write>(
    writer: &mut EventWriter<W>,
) -> Result<(), XmlError> {
    let elem = XmlEvent::end_element();
    writer.write(XmlEvent::from(elem))
}

pub fn parse_translation_unit(source_file_path: String) -> Result<(), AstXmlError> {
    let mut writer = EmitterConfig::new()
        .perform_indent(true)
        .create_writer(std::io::stdout());

    let clang = Clang::new().unwrap();
    let index = Index::new(&clang, false, false);

    let parser = index.parser(source_file_path);
    let tu: TranslationUnit = parser.parse().or_else(|e| Err(AstXmlError::Clang(e)))?;
    let root_entity = tu.get_entity();

    let mut breadcrumbs = vec![root_entity];
    let mut error = Ok(());
    create_start_xml_event_from_entry(root_entity, &mut writer)?;
    root_entity.visit_children(|current, parent| {
        loop {
            let crumb_tail = breadcrumbs.pop().unwrap();
            if crumb_tail == parent {
                breadcrumbs.push(crumb_tail);
                break;
            } else {
                if let Err(e) = create_end_xml_event_from_entry(&mut writer) {
                    error = Err(e);
                    return EntityVisitResult::Break;
                }
            }
        }
        breadcrumbs.push(current);
        create_start_xml_event_from_entry(current, &mut writer)
            .and(Ok(EntityVisitResult::Recurse))
            .or_else(|e| -> Result<EntityVisitResult, XmlError> {
                error = Err(e);
                Ok(EntityVisitResult::Break)
            })
            .unwrap()
    });
    error?;
    while let Some(_entity) = breadcrumbs.pop() {
        create_end_xml_event_from_entry(&mut writer)?;
    }

    Ok(())
}

fn main() {
    let ast_file = env::args().nth(1).expect("1 argument 'ast file' required");

    match parse_translation_unit(ast_file) {
        Ok(_) => (),
        Err(_) => {
            eprintln!("Error");
        }
    }
}
