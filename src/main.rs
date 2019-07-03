use core::result::Result;
use std::env;

use clang::{Clang, Entity, EntityVisitResult, Index, TranslationUnit};

pub fn parse_translation_unit(source_file_path: String) -> Result<(), ()> {
    let clang = Clang::new().unwrap();
    let index = Index::new(&clang, false, false);

    let parser = index.parser(source_file_path);
    let tu: TranslationUnit = parser.parse().or(Err(()))?;
    let entity = tu.get_entity();

    fn walk_through(current: Entity, parent: Entity) -> EntityVisitResult {
        println!("current:{:?}, parent:{:?}", current, parent);

        current.visit_children(&walk_through);

        EntityVisitResult::Continue
    }

    entity.visit_children(&walk_through);

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
