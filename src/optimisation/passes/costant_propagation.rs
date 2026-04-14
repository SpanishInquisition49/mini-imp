use std::collections::HashMap;

use crate::data_flow::annotations::LivenessAnnotation;
use crate::data_flow::graph_schema::Node;
use crate::{
    data_flow::{
        annotations::{AvailableExprAnnotation, VeryBusyExprAnnotation},
        control_flow_graph::ControlFlowGraph,
        graph_schema::{Code, NodeId},
    },
    optimisation::pass::{OptimisationPass, PassResult},
    register_pass,
};

pub struct ConstantPropagation;

register_pass!(ConstantPropagation {
    requires: [AvailableExprAnnotation,],
    modifies: [
        AvailableExprAnnotation,
        VeryBusyExprAnnotation,
        LivenessAnnotation
    ],
});

impl OptimisationPass for ConstantPropagation {
    fn name(&self) -> &'static str {
        "Constant Propagation"
    }

    fn apply(&self, cfg: &mut ControlFlowGraph) -> PassResult {
        let mut changes = 0;
        let node_ids: Vec<NodeId> = cfg.nodes.keys().copied().collect();
        for node_id in node_ids {
            if let Some(node) = cfg.nodes.get_mut(&node_id) {
                let constant_map = Self::build_constant_map_for_node(node);
                match &node.code {
                    Code::Assign(v, exp) => {
                        if let Some(exp_p) = exp.propagate_const(&constant_map) {
                            changes += 1;
                            node.code = Code::Assign(v.clone(), Box::new(exp_p));
                        }
                    }
                    Code::Guard(bexp) => {
                        if let Some(bexp_p) = bexp.propagate_const(&constant_map) {
                            changes += 1;
                            node.code = Code::Guard(Box::new(bexp_p))
                        }
                    }
                    _ => (),
                }
            }
        }

        PassResult {
            changes,
            optimisation_kind: self.name().to_string(),
        }
    }
}

impl ConstantPropagation {
    fn build_constant_map_for_node(node: &Node) -> HashMap<String, i64> {
        let mut constant_map = HashMap::new();

        if let Some(avail_expr) = node.get_annotation::<AvailableExprAnnotation>() {
            for ext_expr in &avail_expr.r#in {
                if let Some(c) = ext_expr.rh_side.extract_const() {
                    constant_map.insert(ext_expr.lh_side.clone(), c);
                }
            }
        }

        constant_map
    }
}
