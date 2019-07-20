use core::result::Result;

use clang::SourceError;
use clang::{Clang, Entity, EntityVisitResult, Index, TranslationUnit};
use clap::{App, AppSettings, Arg};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io::Write;
use xml::writer::Error as XmlError;
use xml::writer::XmlEvent;
use xml::{EmitterConfig, EventWriter};

#[macro_use]
extern crate clap;

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
    let usr = entry
        .get_usr()
        .and_then(|usr| Some(usr.0))
        .unwrap_or_default();
    let src = entry
        .get_location()
        .and_then(|src_loc| Some(src_loc.get_spelling_location()))
        .and_then(|loc| {
            Some(format!(
                "{}:{}:{}:{}",
                loc.file
                    .and_then(|f| f.get_path().into_os_string().into_string().ok())
                    .unwrap_or_default(),
                loc.line,
                loc.column,
                loc.offset
            ))
        })
        .unwrap_or_default();
    let display_name = entry.get_display_name().unwrap_or_default();

    let mut elem = XmlEvent::start_element(kind.as_str());
    if !usr.is_empty() {
        elem = elem.attr("usr", usr.as_str());
    }
    if !src.is_empty() {
        elem = elem.attr("src", src.as_str());
    }
    if !display_name.is_empty() {
        elem = elem.attr("display_name", display_name.as_str());
    }

    writer.write(XmlEvent::from(elem))
}

pub fn create_end_xml_event_from_entry<W: Write>(
    writer: &mut EventWriter<W>,
) -> Result<(), XmlError> {
    let elem = XmlEvent::end_element();
    writer.write(XmlEvent::from(elem))
}

pub fn parse_translation_unit(
    source_file_path: &str,
    arguments: &[&str],
) -> Result<(), AstXmlError> {
    let mut writer = EmitterConfig::new()
        .perform_indent(true)
        .create_writer(std::io::stdout());

    let clang = Clang::new().unwrap();
    let index = Index::new(&clang, false, false);

    let tu: TranslationUnit = {
        let mut parser = index.parser(source_file_path);
        parser.arguments(arguments);
        parser.parse().or_else(|e| Err(AstXmlError::Clang(e)))?
    };
    let root_entity = tu.get_entity();

    let mut breadcrumbs = vec![root_entity];
    let mut error = Ok(());
    create_start_xml_event_from_entry(root_entity, &mut writer)?;
    root_entity.visit_children(|current, parent| {
        match (|| -> Result<(), XmlError> {
            loop {
                let crumb_tail = breadcrumbs.pop().unwrap();
                if crumb_tail == parent {
                    breadcrumbs.push(crumb_tail);
                    break;
                } else {
                    create_end_xml_event_from_entry(&mut writer)?;
                }
            }

            breadcrumbs.push(current);
            create_start_xml_event_from_entry(current, &mut writer)?;
            Ok(())
        })() {
            Ok(_) => EntityVisitResult::Recurse,
            Err(err) => {
                error = Err(err);
                EntityVisitResult::Break
            }
        }
    });
    error?;
    while let Some(_entity) = breadcrumbs.pop() {
        create_end_xml_event_from_entry(&mut writer)?;
    }

    Ok(())
}

fn main() {
    let matches = (app_from_crate!() as App)
        .setting(AppSettings::TrailingVarArg)
        //        .arg(
        //            Arg::with_name("output")
        //                .short("o")
        //                .long("output")
        //                .value_name("OUTPUT")
        //                .help("Sets the XML output file")
        //                .takes_value(true),
        //        )
        .arg(
            Arg::with_name("INPUT")
                .help("input file (source file) and trailing compiler arguments")
                .required(true)
                .multiple(true),
        )
        .get_matches();

    let mut input = matches.values_of("INPUT").unwrap().collect::<Vec<_>>();
    let source_file_path = input.pop().unwrap();
    let arguments = &input[..];
    //let output = matches.value_of("OUTPUT");

    match parse_translation_unit(source_file_path, arguments) {
        Ok(_) => (),
        Err(_) => {
            eprintln!("Error");
        }
    }
}
