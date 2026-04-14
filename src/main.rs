use std::{
    env,
    fs::{self},
    path::PathBuf,
    process::exit,
    str::FromStr,
    sync::Arc,
};

use anyhow::Result as AnyhowResult;

use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::{
    Parser,
    input::{Input, Stream},
    span::SimpleSpan,
};
use logos::Logos;

use crate::{
    data_flow::{
        annotations::{
            AvailableExprAnnotation, DefinedVarsAnnotation, DominatorAnnotation,
            LivenessAnnotation, ReachingDefAnnotation, VeryBusyExprAnnotation,
        },
        code_analysis::{
            available_expr, check_undefined, defined, dominators, liveness, reaching,
            very_busy_expr,
        },
        control_flow_graph::ControlFlowGraph,
    },
    modules::{lexer::Token, parser::parser},
    optimisation::{
        passes::{
            costant_folding::ConstantFolding, costant_propagation::ConstantPropagation,
            dead_code::DeadCodeElimination,
        },
        pipeline::OptimisationPipeline,
    },
};

mod ast;
mod data_flow;
mod modules;
mod optimisation;

#[tokio::main]
async fn main() -> AnyhowResult<()> {
    // NOTE: PHASE 0: PARSE ARGUMENTS
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("Usage: mini-imp <program> <input>");
        std::process::exit(1);
    }

    let path = PathBuf::from_str(&args[1].clone()).unwrap();
    let file_name = path.to_string_lossy().to_string();

    let program = match fs::exists(&path) {
        Ok(true) => fs::read_to_string(&path).unwrap(),
        _ => {
            println!("File '{}' not found.", &path.to_string_lossy());
            std::process::exit(1);
        }
    };

    // NOTE: PHASE 1: LEXER PASS
    let input: i64 = args[2].parse().unwrap();

    let token_iterator = Token::lexer(&program)
        .spanned()
        .map(|(tok, span)| match tok {
            Ok(tok) => (tok, SimpleSpan::from(span)),
            Err(()) => (Token::Error, span.into()),
        });

    let token_stream =
        Stream::from_iter(token_iterator).map((0..program.len()).into(), |(t, s): (_, _)| (t, s));

    // NOTE: PHASE 2: PARSING PASS
    let parsed = parser().parse(token_stream).into_result();
    if let Err(errors) = parsed {
        for err in errors {
            Report::build(ReportKind::Error, (&file_name, err.span().into_range()))
                .with_config(ariadne::Config::new().with_index_type(ariadne::IndexType::Byte))
                .with_code(3)
                .with_message(err.to_string())
                .with_label(
                    Label::new((&file_name, err.span().into_range()))
                        .with_message(err.reason().to_string())
                        .with_color(Color::Red),
                )
                .finish()
                .eprint((&file_name, Source::from(&program)))
                .unwrap();
        }
        exit(1);
    }

    let p = parsed.unwrap();
    let mut cfg = ControlFlowGraph::from(&p);

    // NOTE: PHASE 3: CODE ANALYSIS
    compute_all_annotations(&mut cfg, p.input.clone(), p.output.clone()).await?;
    let mut dot_path = path.clone();
    dot_path.set_extension("dot");
    match fs::write(&dot_path, cfg.to_dot()) {
        Ok(_) => println!("Saved CFG to {}", dot_path.to_string_lossy()),
        Err(e) => println!("Failed to save CFG to {}: {e}", dot_path.to_string_lossy()),
    }

    // NOTE: Check for undefined variables, if any the compilation will stop after reporting them
    // to the user
    match check_undefined(&mut cfg, p.input.clone()) {
        Ok(_) => (),
        Err(errors) => {
            // Report each variable with all locations
            for err in errors {
                Report::build(
                    ReportKind::Error,
                    (&file_name, err.locations[0].into_range()),
                )
                .with_code(4)
                .with_message(format!("Variable '{}' is undefined", err.var_name))
                .with_labels(err.locations.into_iter().map(|span| {
                    Label::new((&file_name, span.into_range()))
                        .with_message("used here")
                        .with_color(Color::Red)
                }))
                .finish()
                .eprint((&file_name, Source::from(&program)))
                .unwrap();
            }
            exit(1);
        }
    }
    // NOTE: PHASE 4: Run the optimisation pipeline
    let mut pipeline = OptimisationPipeline::new(p.input.clone(), p.output.clone());
    pipeline.add_pass(ConstantFolding);
    pipeline.add_pass(ConstantPropagation);
    pipeline.add_pass(DeadCodeElimination);
    println!("Running Optimisation Pipeline");

    for iteration in 1..=10 {
        println!("##### Iteration: {} #####", iteration);
        let changes = pipeline.run(&mut cfg).await?;
        if changes == 0 {
            println!("Reached a fixed point");
            break;
        }
    }

    // NOTE: We recompute all annotations since some of them could be dirty
    // this could be avoided if we don't want to produce the dot file for the optimised CFG
    compute_all_annotations(&mut cfg, p.input.clone(), p.output.clone()).await?;
    let mut optimised_path = path.clone();
    let filename = format!(
        "{}_optimised.dot",
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output")
    );
    optimised_path.set_file_name(filename);
    fs::write(&optimised_path, cfg.to_dot())?;
    println!("Optimized CFG saved to: {}", optimised_path.display());
    // NOTE: PHASE 5: Run the code (Interpretation)
    match p.eval(input) {
        Ok(out) => println!("{out}"),
        Err(err) => println!("Runtime Error: {err}"),
    }

    Ok(())
}

async fn compute_all_annotations(
    cfg: &mut ControlFlowGraph,
    input: String,
    output: String,
) -> AnyhowResult<()> {
    let cfg_ref = Arc::new(cfg.clone());
    let (dom, live, def, reach, avail_exp, busy_exp) = tokio::join!(
        tokio::spawn({
            let cfg = cfg_ref.clone();
            async move { dominators(&cfg) }
        }),
        tokio::spawn({
            let cfg = cfg_ref.clone();
            let output = output.clone();
            async move { liveness(&cfg, output) }
        }),
        tokio::spawn({
            let cfg = cfg_ref.clone();
            let input = input.clone();
            async move { defined(&cfg, input) }
        }),
        tokio::spawn({
            let cfg = cfg_ref.clone();
            let input = input.clone();
            async move { reaching(&cfg, input) }
        }),
        tokio::spawn({
            let cfg = cfg_ref.clone();
            async move { available_expr(&cfg) }
        }),
        tokio::spawn({
            let cfg = cfg_ref.clone();
            async move { very_busy_expr(&cfg) }
        })
    );

    cfg.add_annotation::<LivenessAnnotation, _>(live.unwrap());
    cfg.add_annotation::<DefinedVarsAnnotation, _>(def.unwrap());
    cfg.add_annotation::<ReachingDefAnnotation, _>(reach.unwrap());
    cfg.add_annotation::<AvailableExprAnnotation, _>(avail_exp.unwrap());
    cfg.add_annotation::<VeryBusyExprAnnotation, _>(busy_exp.unwrap());
    cfg.add_annotation::<DominatorAnnotation, _>(dom.unwrap());
    Ok(())
}
