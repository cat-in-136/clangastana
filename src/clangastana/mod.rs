use clang::{Clang, Entity, EntityVisitResult, Index, TranslationUnit};
use core::result::Result;
use failure::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use xml::writer::Error as XmlError;
use xml::writer::XmlEvent;
use xml::{EmitterConfig, EventWriter};

pub mod error;

use error::AstFileLoadError;

#[derive(Default, Debug, Clone)]
pub struct AstXmlOption<'a> {
    pub arguments: &'a [String],
    pub skip_function_bodies: bool,
    pub skip_non_main_file: bool,
}

fn create_start_xml_event_from_entry<W: Write>(
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
    let linkage = entry
        .get_linkage()
        .and_then(|linkage| Some(format!("{:?}", linkage)))
        .unwrap_or_default();
    let module = entry
        .get_module()
        .and_then(|m| Some(m.get_full_name()))
        .unwrap_or_default();
    let (type_kind, type_display_name) = entry
        .get_type()
        .and_then(|t| {
            Some((
                format!("{:?}", t.get_kind()).to_string(),
                t.get_display_name(),
            ))
        })
        .unwrap_or_default();
    let enum_constant_value = entry
        .get_enum_constant_value()
        .and_then(|(_v1, v2)| Some(format!("{}", v2).to_string()))
        .unwrap_or_default();
    let comment = entry.get_comment().unwrap_or_default();
    let display_name = entry.get_display_name().unwrap_or_default();

    let mut elem = XmlEvent::start_element(kind.as_str());

    macro_rules! add_attr {
        ($val: expr) => {
            if !$val.is_empty() {
                elem = elem.attr(stringify!($val), $val.as_str());
            }
        };
    }
    add_attr!(usr);
    add_attr!(src);
    add_attr!(linkage);
    add_attr!(module);
    add_attr!(type_kind);
    add_attr!(type_display_name);
    add_attr!(enum_constant_value);
    add_attr!(comment);
    add_attr!(display_name);

    writer.write(XmlEvent::from(elem))
}

fn create_end_xml_event_from_entry<W: Write>(writer: &mut EventWriter<W>) -> Result<(), XmlError> {
    let elem = XmlEvent::end_element();
    writer.write(XmlEvent::from(elem))
}

pub fn parse_translation_unit<W: Write>(
    tu: TranslationUnit,
    mut writer: &mut EventWriter<W>,
    option: AstXmlOption,
) -> Result<(), Error> {
    let root_entity = tu.get_entity();

    let mut breadcrumbs = vec![root_entity];
    let mut error = Ok(());

    let is_skipped = |entity: Entity| (option.skip_non_main_file && !entity.is_in_main_file());

    create_start_xml_event_from_entry(root_entity, &mut writer)?;
    root_entity.visit_children(|current, parent| {
        match (|| -> Result<(), XmlError> {
            loop {
                let crumb_tail = breadcrumbs.pop().unwrap();
                if crumb_tail == parent {
                    breadcrumbs.push(crumb_tail);
                    break;
                } else {
                    if !is_skipped(crumb_tail) {
                        create_end_xml_event_from_entry(&mut writer)?;
                    }
                }
            }

            breadcrumbs.push(current);
            if !is_skipped(current) {
                create_start_xml_event_from_entry(current, &mut writer)?;
            }
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

pub fn process_astxml(
    source_file_path: String,
    output_file_path: Option<String>,
    option: AstXmlOption,
) -> Result<(), Error> {
    let clang = Clang::new().unwrap();
    let index = Index::new(&clang, false, false);
    let tu: TranslationUnit = if source_file_path.ends_with(".ast") {
        TranslationUnit::from_ast(&index, source_file_path.clone())
            .or(Err(AstFileLoadError::new(source_file_path.clone())))?
    } else {
        let mut parser = index.parser(source_file_path);
        parser.arguments(option.arguments);
        parser.skip_function_bodies(option.skip_function_bodies);
        parser.parse()?
    };

    let output = if let Some(path) = output_file_path {
        Box::new(File::create(path)?) as Box<Write>
    } else {
        Box::new(std::io::stdout()) as Box<Write>
    };
    let mut writer = EmitterConfig::new()
        .perform_indent(true)
        .create_writer(BufWriter::new(output));

    parse_translation_unit(tu, &mut writer, option)
}

#[cfg(test)]
mod tests {
    use std::fs::{read_to_string, File};
    use std::io::Write;
    use std::process::Command;
    use tempfile::tempdir;

    const C_SOURCE: &'static str = r##"int main(void) {
  return 0;
}
"##;
    fn xml_source(c_file_path: &str) -> String {
        format!(r##"<?xml version="1.0" encoding="utf-8"?>
<TranslationUnit display_name="{c_source}">
  <FunctionDecl usr="c:@F@main" src="{c_source}:1:5:4" linkage="External" type_kind="FunctionPrototype" type_display_name="int (void)" display_name="main()">
    <CompoundStmt src="{c_source}:1:16:15">
      <ReturnStmt src="{c_source}:2:3:19">
        <IntegerLiteral src="{c_source}:2:10:26" type_kind="Int" type_display_name="int" />
      </ReturnStmt>
    </CompoundStmt>
  </FunctionDecl>
</TranslationUnit>"##, c_source=c_file_path)
    }

    #[test]
    fn test_process_astxml_c_file() {
        let dir = tempdir().unwrap();
        let c_file_path = dir.path().join("foo.c");
        let mut c_file = File::create(c_file_path.clone()).unwrap();
        let xml_file_path = dir.path().join("foo.xml");

        c_file.write_all(C_SOURCE.as_bytes()).unwrap();

        super::process_astxml(
            String::from(c_file_path.to_str().unwrap()),
            Some(String::from(xml_file_path.to_str().unwrap())),
            Default::default(),
        )
        .unwrap();

        assert_eq!(
            read_to_string(xml_file_path).unwrap(),
            xml_source(c_file_path.to_str().unwrap())
        );

        dir.close().unwrap();
    }

    #[test]
    fn test_process_astxml_ast_file() {
        let dir = tempdir().unwrap();
        let c_file_path = dir.path().join("foo.c");
        let mut c_file = File::create(c_file_path.clone()).unwrap();
        let ast_file_path = dir.path().join("foo.ast");
        let xml_file_path = dir.path().join("foo.xml");

        c_file.write_all(C_SOURCE.as_bytes()).unwrap();

        Command::new("clang")
            .args(&[
                "-o",
                ast_file_path.to_str().unwrap(),
                "-emit-ast",
                c_file_path.to_str().unwrap(),
            ])
            .output()
            .unwrap();

        super::process_astxml(
            String::from(ast_file_path.to_str().unwrap()),
            Some(String::from(xml_file_path.to_str().unwrap())),
            Default::default(),
        )
        .unwrap();

        assert_eq!(
            read_to_string(xml_file_path).unwrap(),
            xml_source(c_file_path.to_str().unwrap())
        );

        dir.close().unwrap();
    }
}
