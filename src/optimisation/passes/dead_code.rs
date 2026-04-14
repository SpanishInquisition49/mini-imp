use crate::data_flow::annotations::{
    AvailableExprAnnotation, ReachingDefAnnotation, VeryBusyExprAnnotation,
};
use crate::data_flow::control_flow_graph::ControlFlowGraph;
use crate::data_flow::graph_schema::{Code, NodeId};
use crate::optimisation::pass::{OptimisationPass, PassResult};
use crate::{data_flow::annotations::LivenessAnnotation, register_pass};

pub struct DeadCodeElimination;

register_pass!(DeadCodeElimination {
    requires: [LivenessAnnotation],
    modifies: [
        ReachingDefAnnotation,
        AvailableExprAnnotation,
        VeryBusyExprAnnotation
    ],
});

impl OptimisationPass for DeadCodeElimination {
    fn name(&self) -> &'static str {
        "Dead Code Elimination"
    }

    fn apply(&self, cfg: &mut ControlFlowGraph) -> PassResult {
        let mut changes = 0;

        let dead_nodes: Vec<NodeId> = cfg
            .nodes
            .iter()
            .filter_map(|(node_id, node)| {
                if let Code::Assign(var, _) = &node.code
                    && let Some(liveness) = node.get_annotation::<LivenessAnnotation>()
                    && !liveness.out.contains(var)
                {
                    return Some(*node_id);
                }
                None
            })
            .collect();

        for node_id in dead_nodes {
            if let Some(node) = cfg.nodes.get_mut(&node_id) {
                node.code = Code::Skip;
                changes += 1;
            }
        }

        // Remove the Skip
        cfg.minimise();

        PassResult {
            changes,
            optimisation_kind: self.name().to_string(),
        }
    }
}
