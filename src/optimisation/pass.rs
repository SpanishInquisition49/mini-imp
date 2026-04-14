use crate::data_flow::{
    annotations::{AnnotationCheck, AnnotationMark},
    control_flow_graph::ControlFlowGraph,
};

pub struct PassResult {
    pub changes: usize,
    pub optimisation_kind: String,
}

pub trait PassMetadata {
    fn get_required_annotations(&self) -> Vec<AnnotationCheck>;
    fn get_modified_annotations(&self) -> Vec<AnnotationMark>;
}

pub trait OptimisationPass: PassMetadata + Send + Sync {
    fn name(&self) -> &'static str;

    fn apply(&self, cfg: &mut ControlFlowGraph) -> PassResult;
}
