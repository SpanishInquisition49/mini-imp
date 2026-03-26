use std::{
    any::{Any, TypeId},
    collections::HashSet,
    ops::Deref,
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

/// Generic in/out annotation — all analyses share this structure.
#[derive(Clone, Debug)]
pub struct Annotation<T> {
    pub r#in: T,
    pub out: T,
}

pub trait AnnotationItem:
    std::fmt::Display + std::fmt::Debug + AnnotationClone + Any + Send + Sync
{
    fn as_any(&self) -> &dyn Any;
    fn get_in(&self) -> &dyn Any;
    fn get_out(&self) -> &dyn Any;
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

/// Implements AnnotationItem, Deref, From<Annotation<T>>, and Display
/// for a newtype wrapper around Annotation<T>.
macro_rules! impl_annotation {
    ($t:ty, $data_type:ty, $display_name:expr) => {
        impl Deref for $t {
            type Target = Annotation<$data_type>;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl From<Annotation<$data_type>> for $t {
            fn from(a: Annotation<$data_type>) -> Self {
                Self(a)
            }
        }

        impl AnnotationItem for $t {
            fn as_any(&self) -> &dyn Any {
                self
            }
            fn get_in(&self) -> &dyn Any {
                &self.0.r#in
            }
            fn get_out(&self) -> &dyn Any {
                &self.0.out
            }
        }

        impl std::fmt::Display for $t {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{}\nin: {}\nout: {}",
                    $display_name,
                    format_set(&self.0.r#in),
                    format_set(&self.0.out),
                )
            }
        }
    };
}

#[derive(Clone, Debug)]
pub struct LivenessAnnotation(pub Annotation<HashSet<String>>);

#[derive(Clone, Debug)]
pub struct DefinedVarsAnnotation(pub Annotation<HashSet<String>>);

#[derive(Clone, Debug)]
pub struct ReachingDefAnnotation(pub Annotation<HashSet<ReachingDefItem>>);

#[derive(Clone, Debug)]
pub struct AvailableExprAnnotation(pub Annotation<HashSet<ExtendedExpr>>);

#[derive(Clone, Debug)]
pub struct VeryBusyExprAnnotation(pub Annotation<HashSet<ExtendedExpr>>);

#[derive(Clone, Debug)]
pub struct DominatorAnnotation(pub Annotation<HashSet<NodeId>>);

impl_annotation!(LivenessAnnotation, HashSet<String>, "Live Variables");
impl_annotation!(DefinedVarsAnnotation, HashSet<String>, "Defined Variables");
impl_annotation!(
    ReachingDefAnnotation,
    HashSet<ReachingDefItem>,
    "Reaching Definitions"
);
impl_annotation!(
    AvailableExprAnnotation,
    HashSet<ExtendedExpr>,
    "Available Expressions"
);
impl_annotation!(
    VeryBusyExprAnnotation,
    HashSet<ExtendedExpr>,
    "Very Busy Expressions"
);
impl_annotation!(DominatorAnnotation, HashSet<NodeId>, "Dominator");

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

    pub fn insert<A: AnnotationItem + Clone + 'static>(&mut self, annotation: A) {
        self.data.insert(TypeId::of::<A>(), Box::new(annotation));
    }

    pub fn get<A: AnnotationItem + 'static>(&self) -> Option<&A> {
        self.data
            .get(&TypeId::of::<A>())
            .and_then(|a| a.as_any().downcast_ref::<A>())
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
