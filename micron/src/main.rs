// use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::process::exit;

use clap::clap_app;
use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use codespan_reporting::term::{self /*ColorArg*/};

mod errors;
mod interpreter;
mod parser;
mod scanner;
mod tokenizer;

#[allow(unused_imports)]
use interpreter::{ErrorInfo, InterpreterError};
use parser::ParseError;
use tokenizer::TokenizerError;

fn file_read(file_name: &str) -> Result<String, Box<dyn Error>> {
    let mut file = File::open(file_name)?;

    let mut content = String::new();
    file.read_to_string(&mut content)?;

    Ok(content)
}

fn main() {
    let matches = clap_app!(micron =>
        (version: "1.0")
        (author: "LoystonLive")
        (about: "A micron rust variant")
        (@arg FILE: +required "Sets the file to run")
        (@arg debug: -d --debug "Print parsing information")
        (@arg pretty: -p --pretty "Prettifies the debug")
        (@arg compileonly: --compileonly "Compiles but doesn't run")
    )
    .get_matches();

    let (source, file_name) = if let Some(file_name) = matches.value_of("FILE") {
        match file_read(file_name) {
            Ok(source) => (source, file_name),
            Err(err) => {
                eprintln!("error: {}", err);
                exit(1);
            }
        }
    } else {
        eprintln!("error: no file provided");
        exit(1);
    };

    let allow_debug = matches.is_present("debug");
    let prettify = matches.is_present("pretty");
    let compile_only = matches.is_present("compileonly");

    macro_rules! debug {
        ($e:expr) => {
            if allow_debug {
                if prettify {
                    println!("{:#?}", $e);
                } else {
                    println!("{:?}", $e);
                }
            }
        };
    }

    let mut files = SimpleFiles::new();
    let file_id = files.add(file_name, &source);
    let source_chars = scanner::Char::from_source(&source);
    let token_infos = tokenizer::tokenize(source_chars);

    debug!(&token_infos);

    let token_infos = match token_infos {
        Ok(ok) => ok,

        Err(err) => {
            match err {
                TokenizerError::SyntaxError(ref info) => {
                    let diagnostic = Diagnostic::error()
                        .with_message(format!("{}", &err))
                        .with_code(format!("{}", &err.error_code()))
                        .with_labels(vec![Label::primary(file_id, info.start..info.end)
                            .with_message(
                                info.msg.as_ref().unwrap_or(&"Invalid syntax".to_string()),
                            )]);

                    let writer = StandardStream::stderr(ColorChoice::Always);
                    let config = codespan_reporting::term::Config::default();

                    term::emit(&mut writer.lock(), &config, &files, &diagnostic).unwrap();
                }
            }
            exit(1);
        }
    };

    let instr_infos = parser::parse(token_infos);

    debug!(&instr_infos);

    let (labels, instr_infos) = match instr_infos {
        Ok(ok) => ok,

        Err(parse_error_info) => {
            let parse_error = &parse_error_info.error;
            let line = &parse_error_info.line;
            let note = &parse_error_info.note;
            let start = line[0].start;
            let end = line[line.len() - 1].end;

            let mut labels = Vec::new();

            match parse_error {
                ParseError::LabelAlreadySet {
                    label: label_string,
                    line: line_at,
                } => {
                    let line_at_start = line_at[0].start;
                    let line_at_end = line_at[line_at.len() - 1].end;

                    labels.push(Label::primary(file_id, start..end).with_message(format!(
                        "Found multiple definitions of label `{}`",
                        &label_string
                    )));

                    labels.push(
                        Label::secondary(file_id, line_at_start..line_at_end).with_message(
                            format!("The label `{}` is already defined here", &label_string),
                        ),
                    );
                }

                ParseError::UnexpectedToken(token_info) => {
                    let start = token_info.start;
                    let end = token_info.end;
                    let token = &token_info.token;

                    labels.push(
                        Label::primary(file_id, start..end)
                            .with_message(format!("Unexpected token `{}`", token)),
                    );
                }

                ParseError::InvalidSyntax => {
                    labels.push(Label::primary(file_id, start..end).with_message("Invalid syntax"));
                }

                ParseError::NotEnoughArgument {
                    token_info,
                    got,
                    expected,
                } => {
                    let start = token_info.start;
                    let end = token_info.end;
                    let _token = &token_info.token;

                    labels.push(Label::primary(file_id, start..end).with_message(format!(
                        "Function `{}` expected {} arguments, got {}",
                        token_info.token, expected, got
                    )));
                }

                ParseError::UnknownFunctionName(token_info) => {
                    let start = token_info.start;
                    let end = token_info.end;
                    let _token = &token_info.token;

                    labels.push(
                        Label::primary(file_id, start..end)
                            .with_message(format!("Unknown function name `{}`", token_info.token)),
                    );
                }
            };

            let notes = match note {
                Some(s) => vec![s.to_string()],
                None => vec![],
            };

            let diagnostic = Diagnostic::error()
                .with_message(format!("{}", parse_error))
                .with_code(format!("{}", parse_error.error_code()))
                .with_labels(labels)
                .with_notes(notes);

            let writer = StandardStream::stderr(ColorChoice::Always);
            let config = codespan_reporting::term::Config::default();

            term::emit(&mut writer.lock(), &config, &files, &diagnostic).unwrap();
            exit(1);
        }
    };

    if !compile_only {
        let result = interpreter::interpret(labels, instr_infos);

        match result {
            Ok(_) => {}
            Err(interpreter_error) => {
                let error_info = &interpreter_error.error_info;
                let instr_info = &interpreter_error.instr_info;
                let fun = &error_info.fun;

                let start = instr_info.start;
                let end = instr_info.end;

                let label_msg = match &error_info.error {
                    interpreter::Error::TypeError { expected, got } => {
                        format!("Function `{}` expected {} got {}", fun, &expected, &got)
                    }
                    interpreter::Error::LabelError(s) => {
                        format!("Got a jump signal to an undefined label `{}`", &s)
                    }
                    interpreter::Error::NoSlotError => format!("No empty slot found"),
                    interpreter::Error::ValueError(val) => {
                        format!("Function `{}`, {} is a bad value", fun, val)
                    }
                    interpreter::Error::Error(err) => format!("Err: {}", err),
                };

                let label = Label::primary(file_id, start..end).with_message(label_msg);

                let notes = match &error_info.note {
                    Some(s) => vec![s.to_string()],
                    None => vec![],
                };

                let diagnostic = Diagnostic::error()
                    .with_message(format!("{}", &error_info.error))
                    .with_code(format!("{}", &error_info.error.error_code()))
                    .with_labels(vec![label])
                    .with_notes(notes);

                let writer = StandardStream::stderr(ColorChoice::Always);
                let config = codespan_reporting::term::Config::default();

                term::emit(&mut writer.lock(), &config, &files, &diagnostic).unwrap();
                exit(1);
            }
        }
    }
}
