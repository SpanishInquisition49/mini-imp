use std::collections::HashMap;

use crate::{
    ast::{
        boolean_exp::BoolExpr,
        cmd::{AtomCmd, Cmd},
        expr::Expr,
    },
    modules::program::Program,
};

pub type NodeId = usize;

#[derive(Debug, Clone)]
pub enum Code {
    Skip,
    Assign(String, Box<Expr>),
    Guard(Box<BoolExpr>),
}

#[derive(Debug, Clone)]
pub enum Edge {
    Bottom,
    Next(NodeId),
    Branch(NodeId, NodeId),
}

#[derive(Debug, Clone)]
pub struct Node {
    pub code: Code,
    pub next: Edge,
}

impl Node {
    pub fn is_removable(&self) -> bool {
        match (&self.code, &self.next) {
            (Code::Skip, Edge::Bottom) => false,
            (Code::Skip, Edge::Next(_)) => true,
            // NOTE: this case should be impossible
            (Code::Skip, Edge::Branch(_, _)) => false,
            (_, _) => false,
        }
    }
}

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

    fn add_node(&mut self, code: Code, next: Edge) -> NodeId {
        let id = self.next_id;
        let node = Node { code, next };
        self.nodes.insert(id, node);
        self.next_id += 1;
        id
    }

    /// Build the Sub-CFG graph for the given Cmd
    /// returning a pair (entry, exit) node ids of the Sub-CFG.
    fn build(&mut self, cmd: &Cmd) -> (NodeId, NodeId) {
        match cmd {
            Cmd::Seq(atom_cmd, cmd) => {
                let (e_cmd1, f_cmd1) = self.sub_build(atom_cmd);
                let (e_cmd2, f_cmd2) = self.build(cmd);
                self.nodes.get_mut(&f_cmd1).unwrap().next = Edge::Next(e_cmd2);
                (e_cmd1, f_cmd2)
            }
            Cmd::AtomCmd(atom_cmd) => self.sub_build(atom_cmd),
        }
    }

    fn sub_build(&mut self, cmd: &AtomCmd) -> (NodeId, NodeId) {
        match cmd {
            AtomCmd::Block(cmd) => self.build(cmd),
            AtomCmd::Assign(var, expr) => {
                let id = self.add_node(Code::Assign(var.clone(), expr.clone()), Edge::Bottom);
                (id, id)
            }
            AtomCmd::Ite(guard, true_branch, false_branch) => {
                let (e_true, f_true) = self.sub_build(true_branch);
                let (e_false, f_false) = self.sub_build(false_branch);

                let join = self.add_node(Code::Skip, Edge::Bottom);

                self.nodes.get_mut(&f_true).unwrap().next = Edge::Next(join);
                self.nodes.get_mut(&f_false).unwrap().next = Edge::Next(join);

                let guard_id =
                    self.add_node(Code::Guard(guard.clone()), Edge::Branch(e_true, e_false));
                // We add a skip to preserve the CFG properties
                let start = self.add_node(Code::Skip, Edge::Next(guard_id));

                (start, join)
            }
            AtomCmd::While(guard, body) => {
                let (e_body, f_body) = self.sub_build(body);
                let join = self.add_node(Code::Skip, Edge::Bottom);
                let guard_id =
                    self.add_node(Code::Guard(guard.clone()), Edge::Branch(e_body, join));
                // We add a skip to preserve the CFG properties
                let start = self.add_node(Code::Skip, Edge::Next(guard_id));

                // we replace the Bottom with the start of the loop in the body CFG
                self.nodes.get_mut(&f_body).unwrap().next = Edge::Next(guard_id);
                (start, join)
            }
            AtomCmd::Skip => {
                let id = self.add_node(Code::Skip, Edge::Bottom);
                (id, id)
            }
            AtomCmd::Print(_) => {
                // NOTE:
                // The print command is just for debugging
                // and is ignored in the CFG (we add a skip, could be removed later)
                let id = self.add_node(Code::Skip, Edge::Bottom);
                (id, id)
            }
        }
    }

    fn node_label(&self, id: NodeId) -> String {
        match &self.nodes[&id].code {
            Code::Skip => String::from("skip"),
            Code::Assign(var, expr) => format!("{} := {}", var, expr),
            Code::Guard(bexp) => format!("{}?", bexp),
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
                Edge::Next(succ) => Edge::Next(successor(&self.nodes, succ)),
                Edge::Branch(t, f) => {
                    Edge::Branch(successor(&self.nodes, t), successor(&self.nodes, f))
                }
            };
            self.nodes.get_mut(&id).unwrap().next = next_new;
        }

        // sweep all the removable nodes
        self.nodes
            .retain(|id, node| !node.is_removable() || *id == self.entry || *id == self.r#final);
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
        let (entry, r#final) = cfg.build(&value.body);
        cfg.entry = entry;
        cfg.r#final = r#final;
        cfg.minimise();
        cfg
    }
}
