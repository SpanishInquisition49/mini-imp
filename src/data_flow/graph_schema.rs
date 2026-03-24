use crate::{
    ast::{boolean_exp::BoolExpr, expr::Expr},
    data_flow::annotations::Annotations,
};
use std::collections::HashSet;

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
    pub pred: Option<HashSet<NodeId>>,
    pub annotations: Annotations,
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

    pub fn add_pred(&mut self, id: NodeId) {
        if let Some(ids) = self.pred.as_mut() {
            ids.insert(id);
        } else {
            // NOTE: this case should be impossible
            self.pred = Some(HashSet::from([id]));
        }
    }
}
