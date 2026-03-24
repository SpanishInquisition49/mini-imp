use std::{
    any::{Any, TypeId},
    collections::HashSet,
};

use indexmap::IndexMap;

use crate::{ast::expr::Expr, data_flow::graph_schema::NodeId};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ReachingDefItem {
    pub var: String,
    pub location: NodeId,
}

impl std::fmt::Display for ReachingDefItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.var, self.location)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ExtendedExpr {
    pub lh_side: String,
    pub rh_side: Box<Expr>,
}

impl std::fmt::Display for ExtendedExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.lh_side, self.rh_side)
    }
}

#[derive(Clone, Debug)]
pub struct Annotations {
    pub data: IndexMap<TypeId, Box<dyn AnnotationItem>>,
}

impl Annotations {
    pub fn new() -> Self {
        Annotations {
            data: IndexMap::new(),
        }
    }
}

// NOTE: every annotation must implement this trait
pub trait AnnotationItem: std::fmt::Display + std::fmt::Debug + AnnotationClone + Any {
    fn as_any(&self) -> &dyn Any;
}

pub trait AnnotationClone {
    fn clone_box(&self) -> Box<dyn AnnotationItem>;
}

impl<T: AnnotationItem + Clone + 'static> AnnotationClone for T {
    fn clone_box(&self) -> Box<dyn AnnotationItem> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn AnnotationItem> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

fn format_set<T: std::fmt::Display>(set: &HashSet<T>) -> String {
    if set.is_empty() {
        "∅".to_string()
    } else {
        let mut out = String::from("{ ");
        for item in set {
            out.push_str(&format!("{item} "));
        }
        out.push('}');
        out
    }
}

impl std::fmt::Display for Annotations {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = String::from("\n--- Annotations ---\n");
        for (_, annotation) in &self.data {
            out.push_str(&format!("{annotation}\n"));
        }
        write!(f, "{out}")
    }
}

#[derive(Clone, Debug)]
pub struct LivenessAnnotation {
    pub live_in: HashSet<String>,
    pub live_out: HashSet<String>,
}

impl AnnotationItem for LivenessAnnotation {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl std::fmt::Display for LivenessAnnotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Live Variables\nin: {}\nout: {}",
            format_set(&self.live_in),
            format_set(&self.live_out),
        )
    }
}

#[derive(Clone, Debug)]
pub struct ReachingDefAnnotation {
    pub reach_in: HashSet<ReachingDefItem>,
    pub reach_out: HashSet<ReachingDefItem>,
}

impl AnnotationItem for ReachingDefAnnotation {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl std::fmt::Display for ReachingDefAnnotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Reaching Definitions\nin: {}\nout: {}",
            format_set(&self.reach_in),
            format_set(&self.reach_out),
        )
    }
}

#[derive(Clone, Debug)]
pub struct DefinedVarsAnnotation {
    pub def_in: HashSet<String>,
    pub def_out: HashSet<String>,
}

impl AnnotationItem for DefinedVarsAnnotation {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl std::fmt::Display for DefinedVarsAnnotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Defined Variables\nin: {}\nout: {}",
            format_set(&self.def_in),
            format_set(&self.def_out),
        )
    }
}

#[derive(Clone, Debug)]
pub struct AvailableExprAnnotation {
    pub avail_in: HashSet<ExtendedExpr>,
    pub avail_out: HashSet<ExtendedExpr>,
}

impl AnnotationItem for AvailableExprAnnotation {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl std::fmt::Display for AvailableExprAnnotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Available Expressions\nin: {}\nout: {}",
            format_set(&self.avail_in),
            format_set(&self.avail_out)
        )
    }
}

#[derive(Clone, Debug)]
pub struct VeryBusyExprAnnotation {
    pub busy_in: HashSet<ExtendedExpr>,
    pub busy_out: HashSet<ExtendedExpr>,
}

impl AnnotationItem for VeryBusyExprAnnotation {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl std::fmt::Display for VeryBusyExprAnnotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Very Busy Expressions\nin: {}\nout: {}",
            format_set(&self.busy_in),
            format_set(&self.busy_out)
        )
    }
}

#[derive(Clone, Debug)]
pub struct DominatorAnnotation {
    pub dom_in: HashSet<NodeId>,
    pub dom_out: HashSet<NodeId>,
}

impl AnnotationItem for DominatorAnnotation {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl std::fmt::Display for DominatorAnnotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Dominator\nin: {}\nout: {}",
            format_set(&self.dom_in),
            format_set(&self.dom_out)
        )
    }
}
