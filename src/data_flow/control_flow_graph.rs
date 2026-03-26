use std::{
    collections::{HashMap, HashSet, VecDeque},
    hash::Hash,
};

use crate::{
    ast::cmd::{AtomCmd, Cmd},
    data_flow::{
        annotations::{Annotation, AnnotationItem, Annotations},
        graph_schema::{Code, Edge, Node, NodeId},
    },
    modules::program::Program,
};

#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
    pub nodes: HashMap<NodeId, Node>,
    pub entry: NodeId,
    pub r#final: NodeId,
    next_id: NodeId,
}

impl ControlFlowGraph {
    pub fn new() -> Self {
        ControlFlowGraph {
            nodes: HashMap::new(),
            entry: 0,
            r#final: 0,
            next_id: 0,
        }
    }

    pub fn forward_worklist<T, I, F, M>(
        &self,
        default: T,
        init: I,
        transfer: F,
        meet: M,
    ) -> HashMap<NodeId, (T, T)>
    where
        T: Clone + PartialEq,
        I: Fn(Vec<NodeId>) -> (HashMap<NodeId, T>, HashMap<NodeId, T>),
        F: Fn(NodeId, &Node, &T) -> T,
        M: Fn(&T, &T) -> T,
    {
        let ids: Vec<NodeId> = self.nodes.keys().copied().collect();
        let (mut r#in, mut out) = init(ids);
        let mut worklist: VecDeque<NodeId> = self.nodes.keys().copied().collect();

        while let Some(node_id) = worklist.pop_front() {
            let node = &self.nodes[&node_id];

            let new_in = match &node.pred {
                Some(pred) => pred
                    .iter()
                    .fold(default.clone(), |acc, pred_id| meet(&acc, &out[pred_id])),
                None => r#in[&node_id].clone(),
            };

            let new_out = transfer(node_id, node, &new_in);
            if new_out != out[&node_id] {
                match &node.next {
                    Edge::Bottom => (),
                    Edge::Next(succ) => {
                        worklist.push_back(*succ);
                    }
                    Edge::Branch(t, f) => {
                        worklist.push_back(*t);
                        worklist.push_back(*f);
                    }
                }
            }

            r#in.insert(node_id, new_in);
            out.insert(node_id, new_out);
        }

        self.nodes
            .keys()
            .map(|id| (*id, (r#in[id].clone(), out[id].clone())))
            .collect()
    }

    pub fn backward_worklist<T, I, F, M>(
        &self,
        init: I,
        transfer: F,
        meet: M,
    ) -> HashMap<NodeId, (T, T)>
    where
        T: Clone + PartialEq,
        I: Fn(Vec<NodeId>) -> (HashMap<NodeId, T>, HashMap<NodeId, T>),
        F: Fn(NodeId, &Node, &T) -> T,
        M: Fn(&T, &T) -> T,
    {
        let ids: Vec<NodeId> = self.nodes.keys().copied().collect();
        let (mut r#in, mut out) = init(ids);
        let mut worklist: VecDeque<NodeId> = self.nodes.keys().copied().collect();

        while let Some(node_id) = worklist.pop_front() {
            let node = &self.nodes[&node_id];

            let new_out = match &node.next {
                Edge::Bottom => out[&node_id].clone(),
                Edge::Next(succ) => r#in[succ].clone(),
                Edge::Branch(t, f) => meet(&r#in[t], &r#in[f]),
            };

            let new_in = transfer(node_id, node, &new_out);

            if new_in != r#in[&node_id]
                && let Some(pred) = &node.pred
            {
                for p in pred {
                    worklist.push_back(*p);
                }
            }

            r#in.insert(node_id, new_in);
            out.insert(node_id, new_out);
        }

        self.nodes
            .keys()
            .map(|id| (*id, (r#in[id].clone(), out[id].clone())))
            .collect()
    }

    pub fn add_annotation<A, T>(&mut self, annotation: HashMap<NodeId, (T, T)>)
    where
        A: AnnotationItem + From<Annotation<T>> + Clone + 'static,
        T: Clone,
    {
        for (node_id, (r#in, out)) in annotation {
            self.nodes
                .get_mut(&node_id)
                .unwrap()
                .annotations
                .insert(A::from(Annotation { r#in, out }));
        }
    }

    pub fn create_universe<F, T>(&self, filter_fun: F) -> HashSet<T>
    where
        T: Clone + PartialEq + Eq + Hash,
        F: Fn(&Node) -> Option<T>,
    {
        self.nodes.values().filter_map(filter_fun).collect()
    }

    fn add_node(&mut self, code: Code, next: Edge, pred: Option<HashSet<NodeId>>) -> NodeId {
        let id = self.next_id;
        let annotations = Annotations::new();
        let node = Node {
            code,
            next,
            pred,
            annotations,
        };
        self.nodes.insert(id, node);
        self.next_id += 1;
        id
    }

    /// Build the Sub-CFG graph for the given Cmd
    /// returning a pair (entry, exit) node ids of the Sub-CFG.
    fn build(&mut self, cmd: &Cmd, pred: Option<HashSet<NodeId>>) -> (NodeId, NodeId) {
        match cmd {
            Cmd::Seq(atom_cmd, cmd) => {
                let (e_cmd1, f_cmd1) = self.sub_build(atom_cmd, pred);
                let (e_cmd2, f_cmd2) = self.build(cmd, Some(HashSet::from([f_cmd1])));
                self.nodes.get_mut(&f_cmd1).unwrap().next = Edge::Next(e_cmd2);
                (e_cmd1, f_cmd2)
            }
            Cmd::AtomCmd(atom_cmd) => self.sub_build(atom_cmd, pred),
        }
    }

    fn sub_build(&mut self, cmd: &AtomCmd, pred: Option<HashSet<NodeId>>) -> (NodeId, NodeId) {
        match cmd {
            AtomCmd::Block(cmd) => self.build(cmd, pred),
            AtomCmd::Assign(var, expr) => {
                let id = self.add_node(Code::Assign(var.clone(), expr.clone()), Edge::Bottom, pred);
                (id, id)
            }
            AtomCmd::Ite(guard, true_branch, false_branch) => {
                // We add a skip to preserve the CFG properties
                let start = self.add_node(Code::Skip, Edge::Bottom, pred);
                let (e_true, f_true) = self.sub_build(true_branch, None);
                let (e_false, f_false) = self.sub_build(false_branch, None);
                // We add a skip to preserve the CFG properties
                let join = self.add_node(
                    Code::Skip,
                    Edge::Bottom,
                    Some(HashSet::from([f_true, f_false])),
                );
                let guard_id = self.add_node(
                    Code::Guard(guard.clone()),
                    Edge::Branch(e_true, e_false),
                    Some(HashSet::from([start])),
                );

                // Here we update the next properties of the nodes
                self.nodes.get_mut(&e_true).unwrap().pred = Some(HashSet::from([guard_id]));
                self.nodes.get_mut(&e_false).unwrap().pred = Some(HashSet::from([guard_id]));
                self.nodes.get_mut(&f_true).unwrap().next = Edge::Next(join);
                self.nodes.get_mut(&f_false).unwrap().next = Edge::Next(join);
                self.nodes.get_mut(&start).unwrap().next = Edge::Next(guard_id);

                (start, join)
            }
            AtomCmd::While(guard, body) => {
                // NOTE:
                // this case is more convoluted because we need generate all the nodes
                // and then update the Next and/or Pred

                // We add a skip to preserve the CFG properties
                let start = self.add_node(Code::Skip, Edge::Bottom, pred);
                let (e_body, f_body) = self.sub_build(body, None);
                let join = self.add_node(Code::Skip, Edge::Bottom, None);
                let guard_id = self.add_node(
                    Code::Guard(guard.clone()),
                    Edge::Branch(e_body, join),
                    Some(HashSet::from([start, f_body])),
                );

                // we replace the Bottom with the start of the loop in the body CFG
                self.nodes.get_mut(&start).unwrap().next = Edge::Next(guard_id);
                self.nodes.get_mut(&f_body).unwrap().next = Edge::Next(guard_id);
                self.nodes.get_mut(&join).unwrap().pred = Some(HashSet::from([guard_id]));
                self.nodes.get_mut(&e_body).unwrap().pred = Some(HashSet::from([guard_id]));
                (start, join)
            }
            AtomCmd::Skip => {
                let id = self.add_node(Code::Skip, Edge::Bottom, pred);
                (id, id)
            }
            AtomCmd::Print(_) => {
                // NOTE:
                // The print command is just for debugging
                // and is ignored in the CFG (we add a skip, could be removed later)
                let id = self.add_node(Code::Skip, Edge::Bottom, pred);
                (id, id)
            }
        }
    }

    fn node_label(&self, id: NodeId) -> String {
        let n = &self.nodes[&id];
        match &n.code {
            Code::Skip => format!("<{id}>: [ skip ]\n{}", n.annotations),
            Code::Assign(var, expr) => {
                format!("<{id}>: [ {} := {} ]\n{}", var, expr, n.annotations)
            }
            Code::Guard(bexp) => format!("<{id}>: [ {}? ]\n{}", bexp, n.annotations),
        }
    }

    /// Remove all the superfluous skip from the CFG
    /// preserving the CFG definition
    pub fn minimise(&mut self) {
        // NOTE:
        // Helper function used to get the real successor of a node
        let successor = |nodes: &HashMap<NodeId, Node>, mut id: NodeId| {
            while let Some(node) = nodes.get(&id) {
                if node.is_removable() {
                    if let Edge::Next(next) = node.next {
                        id = next;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
            id
        };

        let ids: Vec<NodeId> = self.nodes.keys().copied().collect();
        for id in ids {
            let next_new = match self.nodes[&id].next.clone() {
                Edge::Bottom => Edge::Bottom,
                Edge::Next(succ) => {
                    let succ_new = successor(&self.nodes, succ);
                    if succ != succ_new {
                        self.nodes.get_mut(&succ_new).unwrap().add_pred(id);
                    }
                    Edge::Next(succ_new)
                }
                Edge::Branch(t, f) => {
                    let t_new = successor(&self.nodes, t);
                    let f_new = successor(&self.nodes, f);
                    if t != t_new {
                        self.nodes.get_mut(&t_new).unwrap().add_pred(id);
                    }
                    if f != f_new {
                        self.nodes.get_mut(&f_new).unwrap().add_pred(id);
                    }
                    Edge::Branch(t_new, f_new)
                }
            };
            self.nodes.get_mut(&id).unwrap().next = next_new;
        }

        // sweep all the removable nodes
        self.nodes
            .retain(|id, node| !node.is_removable() || *id == self.entry || *id == self.r#final);
        let remaining: Vec<NodeId> = self.nodes.keys().copied().collect();
        for (_, n) in self.nodes.iter_mut() {
            // sweep all the predecessors that does not exists anymore
            if let Some(ids) = n.pred.as_mut() {
                ids.retain(|id| remaining.contains(id));
            }
        }
    }

    pub fn to_dot(&self) -> String {
        let mut out = String::new();

        out.push_str("digraph CFG {\n");
        out.push_str("  graph [rankdir=TB, splines=ortho, nodesep=1.2, ranksep=0.9];\n");
        out.push_str("  node [shape=rectangle, style=\"rounded\", fontname=\"Courier New\", fontsize=14, penwidth=1.5, color=\"#3d4f7c\", fontcolor=\"#3d4f7c\", margin=\"0.3,0.2\"];\n");
        out.push_str("  edge [color=\"#3d4f7c\", penwidth=1.2, arrowsize=0.8, fontname=\"Courier New\", fontsize=12, fontcolor=\"#3d4f7c\"];\n");

        out.push_str("  __start [style=invis, width=0, height=0, label=\"\"];\n");
        out.push_str("  __end   [style=invis, width=0, height=0, label=\"\"];\n");
        out.push_str(&format!("  __start -> {};\n", self.entry));
        out.push_str(&format!("  {} -> __end;\n", self.r#final));

        for (id, node) in &self.nodes {
            let label = self.node_label(*id);
            out.push_str(&format!("  {} [label=\"{}\"];\n", id, label));

            match &node.next {
                Edge::Bottom => {}
                Edge::Next(succ) => {
                    // back-edge: se il successore ha id minore è un arco all'indietro (while)
                    let constraint = if *succ < *id { "constraint=false" } else { "" };
                    if constraint.is_empty() {
                        out.push_str(&format!("  {} -> {};\n", id, succ));
                    } else {
                        out.push_str(&format!("  {} -> {} [{}];\n", id, succ, constraint));
                    }
                }
                Edge::Branch(t, f) => {
                    out.push_str(&format!("  {} -> {} [label=\"true\"];\n", id, t));
                    out.push_str(&format!("  {} -> {} [label=\"false\"];\n", id, f));
                }
            }
        }

        out.push_str("}\n");
        out
    }
}

impl From<&Program> for ControlFlowGraph {
    fn from(value: &Program) -> Self {
        let mut cfg = ControlFlowGraph::new();
        let (entry, r#final) = cfg.build(&value.body, None);
        cfg.entry = entry;
        cfg.r#final = r#final;
        cfg.minimise();
        cfg
    }
}
