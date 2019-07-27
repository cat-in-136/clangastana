mod clangastana;

fn main() {
    let mut args = std::env::args().into_iter();
    let program_name = args.next().unwrap();
    let mut arguments = Vec::with_capacity(std::env::args().len());
    let mut output = None;
    let mut input = None;
    while let Some(v) = args.next() {
        if v.eq(&"-o".to_string()) {
            output = Some(
                args.next()
                    .expect("Error: Output file not specified")
                    .clone(),
            );
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
    let arguments = arguments.as_slice();

    match clangastana::process_astxml(source_file_path, arguments, output) {
        Ok(_) => {
            if output_is_stdout {
                println!();
            }
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
            std::process::exit(1);
        }
    }
}
