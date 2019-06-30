use crate::libclang::{Index, TranslationUnit};
use core::result::Result;
use std::env;
use std::ffi::CString;
use std::path::Path;
use std::ptr;

mod libclang;

pub fn parseTranslationUnit<F: AsRef<Path>>(source_file_path: F) -> Result<(), ()> {
    let mut index = Index::from_ptr(unsafe { clang_sys::clang_createIndex(0, 0) })?;

    let file = CString::new(
        source_file_path
            .as_ref()
            .as_os_str()
            .to_os_string()
            .to_str()
            .ok_or(())?,
    )
    .or(Err(()))?;

    let tu = TranslationUnit::from_ptr(unsafe {
        clang_sys::clang_parseTranslationUnit(
            index.ptr,
            file.as_ptr(),
            ptr::null_mut(),
            0,
            ptr::null_mut(),
            0,
            0,
        )
    })?;

    Ok(())
}

fn main() {
    let ast_file = env::args().nth(1).expect("1 argument 'ast file' required");

    match parseTranslationUnit(ast_file) {
        Ok(_) => (),
        Err(_) => {
            eprintln!("Error");
        }
    }
}
