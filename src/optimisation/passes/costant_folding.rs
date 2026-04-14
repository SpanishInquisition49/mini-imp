use crate::{
    data_flow::{
        annotations::{AvailableExprAnnotation, VeryBusyExprAnnotation},
        control_flow_graph::ControlFlowGraph,
        graph_schema::{Code, NodeId},
    },
    optimisation::pass::{OptimisationPass, PassResult},
    register_pass,
};

pub struct ConstantFolding;

register_pass!(ConstantFolding {
    requires: [],
    modifies: [VeryBusyExprAnnotation, AvailableExprAnnotation],
});

impl OptimisationPass for ConstantFolding {
    fn name(&self) -> &'static str {
        "Constant Folding"
    }

    fn apply(&self, cfg: &mut ControlFlowGraph) -> PassResult {
        let mut changes = 0;

        let node_ids: Vec<NodeId> = cfg.nodes.keys().copied().collect();
        for node_id in node_ids {
            if let Some(node) = cfg.nodes.get_mut(&node_id) {
                match &node.code {
                    Code::Skip => (),
                    Code::Assign(v, expr) => {
                        let (exp_f, exp_c) = expr.fold();
                        if exp_c {
                            node.code = Code::Assign(v.clone(), Box::new(exp_f));
                            changes += 1;
                        }
                    }
                    Code::Guard(bexp) => {
                        let (bexp_f, bexp_c) = bexp.fold();
                        if bexp_c {
                            node.code = Code::Guard(Box::new(bexp_f));
                            changes += 1;
                        }
                    }
                }
            }
        }

        PassResult {
            changes,
            optimisation_kind: self.name().to_string(),
        }
    }
}
