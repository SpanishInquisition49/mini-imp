use std::collections::{HashMap, HashSet};

use chumsky::span::SimpleSpan;

use crate::data_flow::{
    annotations::{DefinedVarsAnnotation, ExtendedExpr, ReachingDefItem},
    control_flow_graph::ControlFlowGraph,
    graph_schema::{Code, NodeId},
};

pub enum AnnotationComputeResult {
    Liveness(HashMap<NodeId, (HashSet<String>, HashSet<String>)>),
    DefinedVars(HashMap<NodeId, (HashSet<String>, HashSet<String>)>),
    ReachingDef(HashMap<NodeId, (HashSet<ReachingDefItem>, HashSet<ReachingDefItem>)>),
    AvailableExpr(HashMap<NodeId, (HashSet<ExtendedExpr>, HashSet<ExtendedExpr>)>),
    VeryBusyExpr(HashMap<NodeId, (HashSet<ExtendedExpr>, HashSet<ExtendedExpr>)>),
    Dominators(HashMap<NodeId, (HashSet<NodeId>, HashSet<NodeId>)>),
}

#[derive(Clone, Debug)]
pub struct UndefinedVarError {
    pub var_name: String,
    pub locations: Vec<SimpleSpan>,
}

pub fn dominators(cfg: &ControlFlowGraph) -> HashMap<NodeId, (HashSet<NodeId>, HashSet<NodeId>)> {
    let universe: HashSet<NodeId> = cfg.nodes.keys().copied().collect();
    cfg.forward_worklist(
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
    )
}

pub fn liveness(
    cfg: &ControlFlowGraph,
    output: String,
) -> HashMap<NodeId, (HashSet<String>, HashSet<String>)> {
    cfg.backward_worklist(
        // NOTE: the initialization function that creates the in and out sets for each node
        |ids| {
            let r#in: HashMap<NodeId, HashSet<String>> = ids
                .iter()
                .map(|id| (*id, HashSet::from([output.clone()])))
                .collect();
            let out: HashMap<NodeId, HashSet<String>> = ids
                .iter()
                .map(|id| (*id, HashSet::from([output.clone()])))
                .collect();
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
    )
}

pub fn defined(
    cfg: &ControlFlowGraph,
    input: String,
) -> HashMap<NodeId, (HashSet<String>, HashSet<String>)> {
    let mut universe = cfg.create_universe(|node| {
        if let Code::Assign(var, _) = &node.code {
            Some(var.clone())
        } else {
            None
        }
    });
    universe.insert(input.clone());
    cfg.forward_worklist(
        // NOTE: the input variable is always defined
        universe.clone(),
        |ids| {
            let r#in: HashMap<NodeId, HashSet<String>> = ids
                .iter()
                .map(|id| {
                    if *id == cfg.entry {
                        (*id, HashSet::from([input.clone()]))
                    } else {
                        (*id, universe.clone())
                    }
                })
                .collect();
            let out: HashMap<NodeId, HashSet<String>> = ids
                .iter()
                .map(|id| {
                    if *id == cfg.entry {
                        (*id, HashSet::from([input.clone()]))
                    } else {
                        (*id, universe.clone())
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
        // NOTE: the meet function is the intersection between all pred of the current node
        // the reduce operation is done inside the skeleton
        |a, b| a.intersection(b).cloned().collect(),
    )
}

pub fn reaching(
    cfg: &ControlFlowGraph,
    input: String,
) -> HashMap<NodeId, (HashSet<ReachingDefItem>, HashSet<ReachingDefItem>)> {
    cfg.forward_worklist(
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
    )
}

pub fn available_expr(
    cfg: &ControlFlowGraph,
) -> HashMap<NodeId, (HashSet<ExtendedExpr>, HashSet<ExtendedExpr>)> {
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
    cfg.forward_worklist(
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
    )
}

pub fn very_busy_expr(
    cfg: &ControlFlowGraph,
) -> HashMap<NodeId, (HashSet<ExtendedExpr>, HashSet<ExtendedExpr>)> {
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
    cfg.backward_worklist(
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
    )
}

pub fn check_undefined(
    cfg: &mut ControlFlowGraph,
    input: String,
) -> Result<(), Vec<UndefinedVarError>> {
    if !cfg.has_annotation::<DefinedVarsAnnotation>() {
        // NOTE: We add the defined variable annotation to the CFG
        let def = defined(cfg, input);
        cfg.add_annotation::<DefinedVarsAnnotation, _>(def);
    }

    let mut errors: HashMap<String, Vec<SimpleSpan>> = HashMap::new();

    for (node_id, node) in &cfg.nodes {
        // NOTE: We can safely unwrap
        let defined = node.get_annotation::<DefinedVarsAnnotation>().unwrap();
        let span = cfg.spans.get(node_id).copied();
        match &node.code {
            Code::Assign(_, exp) => {
                for v in exp.vars() {
                    if !defined.r#in.contains(&v) {
                        // NOTE: again, we can unwrap because only skip don't have Span
                        errors.entry(v).or_insert_with(Vec::new).push(span.unwrap());
                    }
                }
            }
            Code::Guard(bexp) => {
                for v in bexp.vars() {
                    if !defined.r#in.contains(&v) {
                        errors.entry(v).or_insert_with(Vec::new).push(span.unwrap());
                    }
                }
            }
            Code::Skip => (),
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        let result = errors
            .into_iter()
            .map(|(var_name, mut locations)| {
                locations.sort_by_key(|s| s.start);
                UndefinedVarError {
                    var_name,
                    locations,
                }
            })
            .collect();
        Err(result)
    }
}
