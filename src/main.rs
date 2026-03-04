use std::{env, fs};

use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::{
    Parser,
    input::{Input, Stream},
    span::SimpleSpan,
};
use logos::Logos;

use crate::modules::{lexer::Token, parser::parser};

mod modules;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("Usage: mini-imp <program> <input>");
        std::process::exit(1);
    }

    let path = args[1].clone();

    let program = match fs::exists(&path) {
        Ok(true) => fs::read_to_string(&path).unwrap(),
        _ => {
            println!("File '{}' not found.", &path);
            std::process::exit(1);
        }
    };

    let input: i64 = args[2].parse().unwrap();

    let token_iterator = Token::lexer(&program)
        .spanned()
        .map(|(tok, span)| match tok {
            Ok(tok) => (tok, SimpleSpan::from(span)),
            Err(()) => (Token::Error, span.into()),
        });

    let token_stream =
        Stream::from_iter(token_iterator).map((0..program.len()).into(), |(t, s): (_, _)| (t, s));

    match parser().parse(token_stream).into_result() {
        Ok(p) => match p.eval(input) {
            Ok(out) => println!("{out}"),
            Err(err) => println!("Runtime Error: {err}"),
        },
        Err(errors) => {
            for err in errors {
                Report::build(ReportKind::Error, ((), err.span().into_range()))
                    .with_config(ariadne::Config::new().with_index_type(ariadne::IndexType::Byte))
                    .with_code(3)
                    .with_message(err.to_string())
                    .with_label(
                        Label::new(((), err.span().into_range()))
                            .with_message(err.reason().to_string())
                            .with_color(Color::Red),
                    )
                    .finish()
                    .eprint(Source::from(&program))
                    .unwrap();
            }
        }
    }
}
