use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
};

use crate::data_flow::{
    annotations::{
        AvailableExprAnnotation, DefinedVarsAnnotation, DominatorAnnotation, ExtendedExpr,
        LivenessAnnotation, ReachingDefAnnotation, ReachingDefItem, VeryBusyExprAnnotation,
    },
    control_flow_graph::ControlFlowGraph,
    graph_schema::{Code, NodeId},
};

pub fn dominators(cfg: &mut ControlFlowGraph) {
    let universe: HashSet<NodeId> = cfg.nodes.keys().copied().collect();
    let dom = cfg.forward_worklist(
        universe.clone(),
        |ids| {
            let r#in: HashMap<NodeId, HashSet<NodeId>> =
                ids.iter().map(|id| (*id, HashSet::from([*id]))).collect();
            let out: HashMap<NodeId, HashSet<NodeId>> =
                ids.iter().map(|id| (*id, universe.clone())).collect();
            (r#in, out)
        },
        |id, _node, dom_in| {
            let mut dom_out = dom_in.clone();
            dom_out.insert(id);
            dom_out
        },
        |a, b| a.intersection(b).cloned().collect(),
    );

    // Add the Annotation to the CFG nodes
    let nodes: Vec<NodeId> = cfg.nodes.keys().copied().collect();
    for node_id in nodes {
        let (dom_in, dom_out) = dom.get(&node_id).unwrap();
        cfg.nodes
            .get_mut(&node_id)
            .unwrap()
            .annotations
            .data
            .insert(
                TypeId::of::<DominatorAnnotation>(),
                Box::new(DominatorAnnotation {
                    dom_in: dom_in.clone(),
                    dom_out: dom_out.clone(),
                }),
            );
    }
}

pub fn liveness(cfg: &mut ControlFlowGraph) {
    let live = cfg.backward_worklist(
        HashSet::new(),
        // NOTE: the initialization function that creates the in and out sets for each node
        |ids| {
            let r#in: HashMap<NodeId, HashSet<String>> =
                ids.iter().map(|id| (*id, HashSet::new())).collect();
            let out: HashMap<NodeId, HashSet<String>> =
                ids.iter().map(|id| (*id, HashSet::new())).collect();
            (r#in, out)
        },
        // NOTE: the transfer function that compute the new live_in of the node
        |_id, node, live_out| {
            let mut live_in = live_out.clone();
            match &node.code {
                Code::Skip => (),
                Code::Assign(var, expr) => {
                    live_in.remove(var);
                    live_in.extend(expr.vars());
                }
                Code::Guard(bool_expr) => {
                    live_in.extend(bool_expr.vars());
                }
            }
            live_in
        },
        // NOTE: the meet function which is the union between in and out of the node
        |a, b| a.union(b).cloned().collect(),
    );

    // Add the Annotation to the CFG nodes
    let nodes: Vec<NodeId> = cfg.nodes.keys().copied().collect();
    for node_id in nodes {
        let (live_in, live_out) = live.get(&node_id).unwrap();
        cfg.nodes
            .get_mut(&node_id)
            .unwrap()
            .annotations
            .data
            .insert(
                TypeId::of::<LivenessAnnotation>(),
                Box::new(LivenessAnnotation {
                    live_in: live_in.clone(),
                    live_out: live_out.clone(),
                }),
            );
    }
}

pub fn defined(cfg: &mut ControlFlowGraph, input: String) {
    println!("Computing defined with in: '{input}'");
    let defined = cfg.forward_worklist(
        // NOTE: the input variable is always defined
        HashSet::new(),
        |ids| {
            let r#in: HashMap<NodeId, HashSet<String>> = ids
                .iter()
                .map(|id| {
                    if *id == cfg.entry {
                        (*id, HashSet::from([input.clone()]))
                    } else {
                        (*id, HashSet::new())
                    }
                })
                .collect();
            let out: HashMap<NodeId, HashSet<String>> = ids
                .iter()
                .map(|id| {
                    if *id == cfg.entry {
                        (*id, HashSet::from([input.clone()]))
                    } else {
                        (*id, HashSet::new())
                    }
                })
                .collect();
            (r#in, out)
        },
        // NOTE: if present we add to the declared variable to the out set
        |_id, node, def_in| {
            let mut def_out = def_in.clone();
            if let Code::Assign(var, _) = &node.code {
                def_out.insert(var.clone());
            }
            def_out
        },
        // NOTE: the meet function is the union between all pred of the current node
        // the reduce operation is done inside the skeleton
        |a, b| a.union(b).cloned().collect(),
    );

    // Add the Annotation to the CFG nodes
    let nodes: Vec<NodeId> = cfg.nodes.keys().copied().collect();
    for node_id in nodes {
        let (def_in, def_out) = defined.get(&node_id).unwrap();
        cfg.nodes
            .get_mut(&node_id)
            .unwrap()
            .annotations
            .data
            .insert(
                TypeId::of::<DefinedVarsAnnotation>(),
                Box::new(DefinedVarsAnnotation {
                    def_in: def_in.clone(),
                    def_out: def_out.clone(),
                }),
            );
    }
}

pub fn reaching(cfg: &mut ControlFlowGraph, input: String) {
    let reaching = cfg.forward_worklist(
        // NOTE: the reaching definition of the input var at the start is defined
        // in the entry block
        HashSet::from([ReachingDefItem {
            var: input.clone(),
            location: cfg.entry,
        }]),
        // NOTE: we start with the empty set for each block
        |ids| {
            let r#in: HashMap<NodeId, HashSet<ReachingDefItem>> =
                ids.iter().map(|id| (*id, HashSet::new())).collect();
            let out: HashMap<NodeId, HashSet<ReachingDefItem>> =
                ids.iter().map(|id| (*id, HashSet::new())).collect();
            (r#in, out)
        },
        |id, node, reach_in| -> HashSet<ReachingDefItem> {
            let mut reach_out = reach_in.clone();
            if let Code::Assign(var, _) = &node.code {
                reach_out.retain(|i| &i.var != var);
                let item = ReachingDefItem {
                    var: var.clone(),
                    location: id,
                };
                reach_out.insert(item);
            }
            reach_out
        },
        // NOTE: the meet operator for the Reaching definitions is the union
        |a, b| a.union(b).cloned().collect(),
    );
    // Add the Annotation to the CFG nodes
    let nodes: Vec<NodeId> = cfg.nodes.keys().copied().collect();
    for node_id in nodes {
        let (reach_in, reach_out) = reaching.get(&node_id).unwrap();
        cfg.nodes
            .get_mut(&node_id)
            .unwrap()
            .annotations
            .data
            .insert(
                TypeId::of::<ReachingDefAnnotation>(),
                Box::new(ReachingDefAnnotation {
                    reach_in: reach_in.clone(),
                    reach_out: reach_out.clone(),
                }),
            );
    }
}

pub fn available_expr(cfg: &mut ControlFlowGraph) {
    let universe: HashSet<ExtendedExpr> = cfg.create_universe(|node| {
        if let Code::Assign(var, expr) = &node.code {
            Some(ExtendedExpr {
                lh_side: var.clone(),
                rh_side: expr.clone(),
            })
        } else {
            None
        }
    });
    let available = cfg.forward_worklist(
        universe.clone(),
        |ids| {
            let r#in: HashMap<NodeId, HashSet<ExtendedExpr>> = ids
                .iter()
                .map(|id| {
                    (
                        *id,
                        if *id == cfg.entry {
                            HashSet::new()
                        } else {
                            universe.clone()
                        },
                    )
                })
                .collect();
            let out: HashMap<NodeId, HashSet<ExtendedExpr>> = cfg
                .nodes
                .keys()
                .map(|id| {
                    (
                        *id,
                        if *id == cfg.entry {
                            HashSet::new()
                        } else {
                            universe.clone()
                        },
                    )
                })
                .collect();
            (r#in, out)
        },
        |_id, node, avail_in| {
            let mut avail_out = avail_in.clone();
            if let Code::Assign(var, expr) = &node.code {
                // NOTE: kill all expressions with the var
                avail_out.retain(|e| &e.lh_side != var && !e.rh_side.vars().contains(var));
                avail_out.insert(ExtendedExpr {
                    lh_side: var.clone(),
                    rh_side: expr.clone(),
                });
            }
            avail_out
        },
        // NOTE: the available expression meet operator is the intersection
        |a, b| a.intersection(b).cloned().collect(),
    );

    // Add the Annotation to the CFG nodes
    let nodes: Vec<NodeId> = cfg.nodes.keys().copied().collect();
    for node_id in nodes {
        let (avail_in, avail_out) = available.get(&node_id).unwrap();
        cfg.nodes
            .get_mut(&node_id)
            .unwrap()
            .annotations
            .data
            .insert(
                TypeId::of::<AvailableExprAnnotation>(),
                Box::new(AvailableExprAnnotation {
                    avail_in: avail_in.clone(),
                    avail_out: avail_out.clone(),
                }),
            );
    }
}

pub fn very_busy_expr(cfg: &mut ControlFlowGraph) {
    let universe: HashSet<ExtendedExpr> = cfg.create_universe(|node| {
        if let Code::Assign(var, expr) = &node.code {
            Some(ExtendedExpr {
                lh_side: var.clone(),
                rh_side: expr.clone(),
            })
        } else {
            None
        }
    });
    let busy = cfg.backward_worklist(
        universe.clone(),
        |ids| {
            let r#in: HashMap<NodeId, HashSet<ExtendedExpr>> = ids
                .iter()
                .map(|id| {
                    (
                        *id,
                        if *id == cfg.r#final {
                            HashSet::new()
                        } else {
                            universe.clone()
                        },
                    )
                })
                .collect();
            let out: HashMap<NodeId, HashSet<ExtendedExpr>> = cfg
                .nodes
                .keys()
                .map(|id| {
                    (
                        *id,
                        if *id == cfg.r#final {
                            HashSet::new()
                        } else {
                            universe.clone()
                        },
                    )
                })
                .collect();
            (r#in, out)
        },
        |_id, node, busy_out| {
            let mut busy_in = busy_out.clone();
            if let Code::Assign(var, expr) = &node.code {
                busy_in.retain(|e| var != &e.lh_side && !e.rh_side.vars().contains(var));
                busy_in.insert(ExtendedExpr {
                    lh_side: var.clone(),
                    rh_side: expr.clone(),
                });
            }
            busy_in
        },
        |a, b| a.intersection(b).cloned().collect(),
    );
    // Add the Annotation to the CFG nodes
    let nodes: Vec<NodeId> = cfg.nodes.keys().copied().collect();
    for node_id in nodes {
        let (busy_in, busy_out) = busy.get(&node_id).unwrap();
        cfg.nodes
            .get_mut(&node_id)
            .unwrap()
            .annotations
            .data
            .insert(
                TypeId::of::<VeryBusyExprAnnotation>(),
                Box::new(VeryBusyExprAnnotation {
                    busy_in: busy_in.clone(),
                    busy_out: busy_out.clone(),
                }),
            );
    }
}
