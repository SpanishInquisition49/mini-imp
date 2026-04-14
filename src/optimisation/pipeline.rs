use anyhow::{Result as AnyhowResult, anyhow};
use std::{any::TypeId, usize};

use crate::{
    data_flow::{
        annotations::{
            AvailableExprAnnotation, DefinedVarsAnnotation, DominatorAnnotation,
            LivenessAnnotation, ReachingDefAnnotation, VeryBusyExprAnnotation,
        },
        code_analysis::AnnotationComputeResult,
        control_flow_graph::ControlFlowGraph,
    },
    optimisation::pass::OptimisationPass,
};

pub struct OptimisationPipeline {
    pub input: String,
    pub output: String,
    pub passes: Vec<Box<dyn OptimisationPass>>,
}

impl OptimisationPipeline {
    pub fn new(input: String, output: String) -> Self {
        OptimisationPipeline {
            input,
            output,
            passes: Vec::new(),
        }
    }

    pub fn add_pass<P: OptimisationPass + 'static>(&mut self, pass: P) {
        self.passes.push(Box::new(pass));
    }

    pub async fn run(&mut self, cfg: &mut ControlFlowGraph) -> AnyhowResult<usize> {
        let mut total_changes = 0;
        for pass in &mut self.passes {
            let required_annotations = pass.get_required_annotations();
            let dirty_annotations: Vec<TypeId> = required_annotations
                .iter()
                .filter(|(_, is_dirty_fn)| is_dirty_fn(cfg))
                .map(|(type_id, _)| *type_id)
                .collect();

            // NOTE: if some required annotations are dirty recompute them
            if !dirty_annotations.is_empty() {
                println!("Found some dirty annotations");
                Self::recompute_annotations(
                    cfg,
                    dirty_annotations,
                    self.input.clone(),
                    self.output.clone(),
                )
                .await?;
            }

            let result = pass.apply(cfg);
            total_changes += result.changes;
            println!(
                "Executed: {} Changes: {}",
                result.optimisation_kind, result.changes
            );

            // NOTE: mark the annotations to dirty only when the CFG changes
            if result.changes > 0 {
                let modified_annotations = pass.get_modified_annotations();
                for (_, mark_dirty) in modified_annotations {
                    mark_dirty(cfg);
                }
            }
        }
        Ok(total_changes)
    }

    async fn recompute_annotations(
        cfg: &mut ControlFlowGraph,
        annotations: Vec<TypeId>,
        input: String,
        output: String,
    ) -> AnyhowResult<()> {
        let mut futures = Vec::new();

        for annotation in annotations {
            let cfg_cloned = cfg.clone();
            let input = input.clone();
            let output = output.clone();
            let fut = tokio::spawn(async move {
                Self::compute_annotation(&cfg_cloned, &annotation, input, output)
            });
            futures.push((annotation, fut));
        }

        for (_, fut) in futures {
            let result = fut.await??;
            Self::add_annotation(cfg, result);
        }

        Ok(())
    }

    fn compute_annotation(
        cfg: &ControlFlowGraph,
        annotation: &TypeId,
        input: String,
        output: String,
    ) -> AnyhowResult<AnnotationComputeResult> {
        use crate::data_flow::code_analysis::*;
        // Dispatch based on TypeId
        if *annotation == TypeId::of::<LivenessAnnotation>() {
            let result = liveness(cfg, output);
            println!("Recomputed: Liveness analysis");
            Ok(AnnotationComputeResult::Liveness(result))
        } else if *annotation == TypeId::of::<DefinedVarsAnnotation>() {
            let result = defined(cfg, input);
            println!("Recomputed: Defined variables analysis");
            Ok(AnnotationComputeResult::DefinedVars(result))
        } else if *annotation == TypeId::of::<ReachingDefAnnotation>() {
            let result = reaching(cfg, input);
            println!("Recomputed: Reaching Definition analysis");
            Ok(AnnotationComputeResult::ReachingDef(result))
        } else if *annotation == TypeId::of::<AvailableExprAnnotation>() {
            let result = available_expr(cfg);
            println!("Recomputed: Available Expr analysis");
            Ok(AnnotationComputeResult::AvailableExpr(result))
        } else if *annotation == TypeId::of::<VeryBusyExprAnnotation>() {
            let result = very_busy_expr(cfg);
            println!("Recomputed: Very Busy Expr analysis");
            Ok(AnnotationComputeResult::VeryBusyExpr(result))
        } else if *annotation == TypeId::of::<DominatorAnnotation>() {
            let result = dominators(cfg);
            println!("Recomputed: Dominators analysis");
            Ok(AnnotationComputeResult::Dominators(result))
        } else {
            Err(anyhow!("Unknown annotation type"))
        }
    }

    fn add_annotation(cfg: &mut ControlFlowGraph, result: AnnotationComputeResult) {
        match result {
            AnnotationComputeResult::Liveness(map) => {
                cfg.add_annotation::<LivenessAnnotation, _>(map);
            }
            AnnotationComputeResult::DefinedVars(map) => {
                cfg.add_annotation::<DefinedVarsAnnotation, _>(map);
            }
            AnnotationComputeResult::ReachingDef(map) => {
                cfg.add_annotation::<ReachingDefAnnotation, _>(map);
            }
            AnnotationComputeResult::AvailableExpr(map) => {
                cfg.add_annotation::<AvailableExprAnnotation, _>(map);
            }
            AnnotationComputeResult::VeryBusyExpr(map) => {
                cfg.add_annotation::<VeryBusyExprAnnotation, _>(map);
            }
            AnnotationComputeResult::Dominators(map) => {
                cfg.add_annotation::<DominatorAnnotation, _>(map);
            }
        }
    }
}
