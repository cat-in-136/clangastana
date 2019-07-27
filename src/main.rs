mod clangastana;
#[macro_use]
extern crate failure;

use crate::clangastana::AstXmlOption;

fn main() {
    let mut args = std::env::args().into_iter();
    let program_name = args.next().unwrap();
    let mut arguments = Vec::with_capacity(std::env::args().len());
    let mut skip_function_bodies = false;
    let mut skip_non_main_file = false;
    let mut output = None;
    let mut input = None;
    while let Some(v) = args.next() {
        if v.eq(&"-o".to_string()) || v.eq(&"--output".to_string()) {
            output = Some(
                args.next()
                    .expect("Error: Output file not specified")
                    .clone(),
            );
        } else if v.eq(&"--skip-function-bodies".to_string()) {
            skip_function_bodies = true;
        } else if v.eq(&"--skip-non-main-file".to_string()) {
            skip_non_main_file = true;
        } else if v.eq(&"-h".to_string()) || v.eq(&"--help".to_string()) {
            println!(
                r#"{description}.

USAGE:
    {program_name} [OPTIONS] <INPUT>...
    {program_name} [OPTIONS] input.ast

FLAGS:
    -h, --help       Prints help information

OPTIONS:
    -o, --output <OUTPUT>    Sets the XML output file
    --skip-function-bodies   Skip function and method bodies
    --skip-non-main-file     Skip non-main file entities

ARGS:
    <INPUT>...    input file (source file) and trailing compiler arguments
"#,
                description = env!("CARGO_PKG_DESCRIPTION"),
                program_name = program_name
            );
            std::process::exit(1);
        } else if input.is_none() && !v.starts_with("-") {
            input = Some(v);
        } else {
            arguments.push(v);
        }
    }
    let output_is_stdout = output.is_none();
    let source_file_path = input.expect("Error: input file not specified").clone();
    let option = AstXmlOption {
        arguments: arguments.as_slice(),
        skip_function_bodies,
        skip_non_main_file,
        ..Default::default()
    };

    match clangastana::process_astxml(source_file_path, output, option) {
        Ok(_) => {
            if output_is_stdout {
                println!();
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
