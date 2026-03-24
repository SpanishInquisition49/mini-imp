use std::{
    env,
    fs::{self},
    path::PathBuf,
    str::FromStr,
};

use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::{
    Parser,
    input::{Input, Stream},
    span::SimpleSpan,
};
use logos::Logos;

use crate::{
    data_flow::{
        code_analysis::{available_expr, defined, dominators, liveness, reaching, very_busy_expr},
        control_flow_graph::ControlFlowGraph,
    },
    modules::{lexer::Token, parser::parser},
};

mod ast;
mod data_flow;
mod modules;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("Usage: mini-imp <program> <input>");
        std::process::exit(1);
    }

    let mut path = PathBuf::from_str(&args[1].clone()).unwrap();

    let program = match fs::exists(&path) {
        Ok(true) => fs::read_to_string(&path).unwrap(),
        _ => {
            println!("File '{}' not found.", &path.to_string_lossy());
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
        Ok(p) => {
            let mut cfg = ControlFlowGraph::from(&p);
            // NOTE: we perform static analysis here
            dominators(&mut cfg);
            liveness(&mut cfg);
            defined(&mut cfg, p.input.clone());
            reaching(&mut cfg, p.input.clone());
            available_expr(&mut cfg);
            very_busy_expr(&mut cfg);
            path.set_extension("dot");
            match fs::write(&path, cfg.to_dot()) {
                Ok(_) => println!("Saved CFG to {}", path.to_string_lossy()),
                Err(e) => println!("Failed to save CFG to {}: {e}", path.to_string_lossy()),
            }
            match p.eval(input) {
                Ok(out) => println!("{out}"),
                Err(err) => println!("Runtime Error: {err}"),
            }
        }
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
