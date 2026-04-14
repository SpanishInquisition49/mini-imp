use core::fmt;
use std::collections::{HashMap, HashSet};

use chumsky::span::SimpleSpan;

use crate::{
    ast::expr::{Expr, Factor, Term},
    modules::eval::{Env, EvalError},
};

enum CmpOp {
    LowerThan,
    GreaterThan,
}

#[derive(Debug, Clone)]
pub enum BoolExpr {
    And(Box<BoolExpr>, Box<Atom>, SimpleSpan),
    Or(Box<BoolExpr>, Box<Atom>, SimpleSpan),
    Not(Box<BoolExpr>, SimpleSpan),
    LowerThan(Box<Expr>, Box<Expr>, SimpleSpan),
    GreaterThan(Box<Expr>, Box<Expr>, SimpleSpan),
    Atom(Box<Atom>, SimpleSpan),
}

impl BoolExpr {
    pub fn eval(&self, env: &Env) -> Result<bool, EvalError> {
        match self {
            BoolExpr::And(l, r, _) => Ok(l.eval(env)? && r.eval(env)?),
            BoolExpr::Or(l, r, _) => Ok(l.eval(env)? || r.eval(env)?),
            BoolExpr::Not(e, _) => Ok(!e.eval(env)?),
            BoolExpr::LowerThan(l, r, _) => Ok(l.eval(env)? < r.eval(env)?),
            BoolExpr::GreaterThan(l, r, _) => Ok(l.eval(env)? > r.eval(env)?),
            BoolExpr::Atom(atom, _) => Ok(atom.eval(env)?),
        }
    }

    pub fn vars(&self) -> HashSet<String> {
        let mut vars = HashSet::new();
        match self {
            BoolExpr::And(bool_expr, atom, _) => {
                vars.extend(bool_expr.vars());
                vars.extend(atom.vars());
            }
            BoolExpr::Or(bool_expr, atom, _) => {
                vars.extend(bool_expr.vars());
                vars.extend(atom.vars());
            }
            BoolExpr::Not(bool_expr, _) => {
                vars.extend(bool_expr.vars());
            }
            BoolExpr::LowerThan(expr, expr1, _) => {
                vars.extend(expr.vars());
                vars.extend(expr1.vars());
            }
            BoolExpr::GreaterThan(expr, expr1, _) => {
                vars.extend(expr.vars());
                vars.extend(expr1.vars());
            }
            BoolExpr::Atom(atom, _) => {
                vars.extend(atom.vars());
            }
        };
        vars
    }

    pub fn span(&self) -> SimpleSpan {
        match self {
            BoolExpr::And(_, _, span) => *span,
            BoolExpr::Or(_, _, span) => *span,
            BoolExpr::Not(_, span) => *span,
            BoolExpr::LowerThan(_, _, span) => *span,
            BoolExpr::GreaterThan(_, _, span) => *span,
            BoolExpr::Atom(_, span) => *span,
        }
    }

    pub fn propagate_const(&self, constant_map: &HashMap<String, i64>) -> Option<BoolExpr> {
        match self {
            BoolExpr::And(left, right, simple_span) => match (
                left.propagate_const(constant_map),
                right.propagate_const(constant_map),
            ) {
                (None, None) => None,
                (None, Some(right_p)) => {
                    Some(BoolExpr::And(left.clone(), Box::new(right_p), *simple_span))
                }
                (Some(left_p), None) => {
                    Some(BoolExpr::And(Box::new(left_p), right.clone(), *simple_span))
                }
                (Some(left_p), Some(right_p)) => Some(BoolExpr::And(
                    Box::new(left_p),
                    Box::new(right_p),
                    *simple_span,
                )),
            },
            BoolExpr::Or(left, right, simple_span) => match (
                left.propagate_const(constant_map),
                right.propagate_const(constant_map),
            ) {
                (None, None) => None,
                (None, Some(right_p)) => {
                    Some(BoolExpr::Or(left.clone(), Box::new(right_p), *simple_span))
                }
                (Some(left_p), None) => {
                    Some(BoolExpr::Or(Box::new(left_p), right.clone(), *simple_span))
                }
                (Some(left_p), Some(right_p)) => Some(BoolExpr::Or(
                    Box::new(left_p),
                    Box::new(right_p),
                    *simple_span,
                )),
            },
            BoolExpr::Not(bool_expr, simple_span) => bool_expr
                .propagate_const(constant_map)
                .map(|bexp| BoolExpr::Not(Box::new(bexp), *simple_span)),
            BoolExpr::LowerThan(left, right, simple_span) => match (
                left.propagate_const(constant_map),
                right.propagate_const(constant_map),
            ) {
                (None, None) => None,
                (None, Some(right_p)) => Some(BoolExpr::LowerThan(
                    left.clone(),
                    Box::new(right_p),
                    *simple_span,
                )),
                (Some(left_p), None) => Some(BoolExpr::LowerThan(
                    Box::new(left_p),
                    right.clone(),
                    *simple_span,
                )),
                (Some(left_p), Some(right_p)) => Some(BoolExpr::LowerThan(
                    Box::new(left_p),
                    Box::new(right_p),
                    *simple_span,
                )),
            },
            BoolExpr::GreaterThan(left, right, simple_span) => match (
                left.propagate_const(constant_map),
                right.propagate_const(constant_map),
            ) {
                (None, None) => None,
                (None, Some(right_p)) => Some(BoolExpr::GreaterThan(
                    left.clone(),
                    Box::new(right_p),
                    *simple_span,
                )),
                (Some(left_p), None) => Some(BoolExpr::GreaterThan(
                    Box::new(left_p),
                    right.clone(),
                    *simple_span,
                )),
                (Some(left_p), Some(right_p)) => Some(BoolExpr::GreaterThan(
                    Box::new(left_p),
                    Box::new(right_p),
                    *simple_span,
                )),
            },
            BoolExpr::Atom(atom, simple_span) => atom
                .propagate_const(constant_map)
                .map(|atom_p| BoolExpr::Atom(Box::new(atom_p), *simple_span)),
        }
    }

    pub fn fold(&self) -> (BoolExpr, bool) {
        match self {
            BoolExpr::And(bool_expr, atom, simple_span) => {
                let (bexp_f, bexp_c) = bool_expr.fold();
                let (atom_f, atom_c) = atom.fold();

                if bexp_f.is_false() {
                    return (BoolExpr::Atom(Box::new(Atom::False), *simple_span), true);
                }

                if atom_f.is_false() {
                    return (BoolExpr::Atom(Box::new(Atom::False), *simple_span), true);
                }

                if bexp_f.is_true() {
                    return (BoolExpr::Atom(Box::new(atom_f), *simple_span), true);
                }

                if atom_f.is_true() {
                    return (bexp_f, true);
                }

                (
                    BoolExpr::And(Box::new(bexp_f), Box::new(atom_f), *simple_span),
                    bexp_c || atom_c,
                )
            }
            BoolExpr::Or(bool_expr, atom, simple_span) => {
                let (bexp_f, bexp_c) = bool_expr.fold();
                let (atom_f, atom_c) = atom.fold();

                if bexp_f.is_false() {
                    return (BoolExpr::Atom(Box::new(atom_f), *simple_span), true);
                }

                if atom_f.is_false() {
                    return (bexp_f, true);
                }

                if bexp_f.is_true() {
                    return (BoolExpr::Atom(Box::new(Atom::True), *simple_span), true);
                }

                if atom_f.is_true() {
                    return (BoolExpr::Atom(Box::new(Atom::True), *simple_span), true);
                }

                (
                    BoolExpr::Or(Box::new(bexp_f), Box::new(atom_f), *simple_span),
                    bexp_c || atom_c,
                )
            }
            BoolExpr::Not(bool_expr, simple_span) => {
                let (bexp_f, bexp_c) = bool_expr.fold();

                if bexp_f.is_false() {
                    return (BoolExpr::Atom(Box::new(Atom::True), *simple_span), true);
                }

                if bexp_f.is_true() {
                    return (BoolExpr::Atom(Box::new(Atom::False), *simple_span), true);
                }

                (BoolExpr::Not(Box::new(bexp_f), *simple_span), bexp_c)
            }
            BoolExpr::LowerThan(left, right, simple_span) => {
                let (left_f, left_c) = left.fold();
                let (right_f, right_c) = right.fold();

                if let Some(result) = Self::try_fold_comparison(&left_f, &right_f, CmpOp::LowerThan)
                {
                    let atom = if result { Atom::True } else { Atom::False };
                    return (BoolExpr::Atom(Box::new(atom), *simple_span), true);
                }

                (
                    BoolExpr::LowerThan(Box::new(left_f), Box::new(right_f), *simple_span),
                    left_c || right_c,
                )
            }
            BoolExpr::GreaterThan(left, right, simple_span) => {
                let (left_f, left_c) = left.fold();
                let (right_f, right_c) = right.fold();

                if let Some(result) =
                    Self::try_fold_comparison(&left_f, &right_f, CmpOp::GreaterThan)
                {
                    let atom = if result { Atom::True } else { Atom::False };
                    return (BoolExpr::Atom(Box::new(atom), *simple_span), true);
                }

                (
                    BoolExpr::GreaterThan(Box::new(left_f), Box::new(right_f), *simple_span),
                    left_c || right_c,
                )
            }
            BoolExpr::Atom(atom, simple_span) => {
                let (atom_f, atom_c) = atom.fold();
                (BoolExpr::Atom(Box::new(atom_f), *simple_span), atom_c)
            }
        }
    }

    fn is_true(&self) -> bool {
        match &self {
            BoolExpr::Atom(a, _) => a.is_true(),
            _ => false,
        }
    }

    fn is_false(&self) -> bool {
        match &self {
            BoolExpr::Atom(a, _) => a.is_false(),
            _ => false,
        }
    }

    fn try_fold_comparison(left: &Expr, right: &Expr, op: CmpOp) -> Option<bool> {
        if let (Expr::Term(t_left), Expr::Term(t_right)) = (left, right)
            && let (Term::Fac(f_left), Term::Fac(f_right)) = (&**t_left, &**t_right)
            && let (Factor::Int(i_left), Factor::Int(i_right)) = (&**f_left, &**f_right)
        {
            match op {
                CmpOp::LowerThan => Some(*i_left < *i_right),
                CmpOp::GreaterThan => Some(*i_left > *i_right),
            }
        } else {
            None
        }
    }
}

impl fmt::Display for BoolExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BoolExpr::And(l, r, _) => write!(f, "{l} and {r}"),
            BoolExpr::Or(l, r, _) => write!(f, "{l} or {r}"),
            BoolExpr::Not(bexp, _) => write!(f, "not {bexp}"),
            BoolExpr::LowerThan(l, r, _) => write!(f, "{l} < {r}"),
            BoolExpr::GreaterThan(l, r, _) => write!(f, "{l} > {r}"),
            BoolExpr::Atom(atom, _) => write!(f, "{atom}"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Atom {
    True,
    False,
    SubBexp(Box<BoolExpr>),
}

impl Atom {
    pub fn eval(&self, env: &Env) -> Result<bool, EvalError> {
        match self {
            Atom::True => Ok(true),
            Atom::False => Ok(false),
            Atom::SubBexp(bexp) => Ok(bexp.eval(env)?),
        }
    }

    pub fn vars(&self) -> HashSet<String> {
        let vars = HashSet::new();
        match self {
            Atom::True => vars,
            Atom::False => vars,
            Atom::SubBexp(bool_expr) => bool_expr.vars(),
        }
    }

    pub fn propagate_const(&self, constant_map: &HashMap<String, i64>) -> Option<Atom> {
        match self {
            Atom::SubBexp(bool_expr) => bool_expr
                .propagate_const(constant_map)
                .map(|bexp_p| Atom::SubBexp(Box::new(bexp_p))),
            _ => None,
        }
    }

    pub fn is_true(&self) -> bool {
        matches!(self, Atom::True)
    }

    pub fn is_false(&self) -> bool {
        matches!(self, Atom::False)
    }

    pub fn fold(&self) -> (Atom, bool) {
        match self {
            Atom::SubBexp(bexp) => {
                let (bexp_f, bexp_c) = bexp.fold();
                // Extract: (true) → true, (false) → false
                if let BoolExpr::Atom(atom, _) = &bexp_f {
                    return (**atom).clone().fold();
                }
                (Atom::SubBexp(Box::new(bexp_f)), bexp_c)
            }
            _ => (self.clone(), false),
        }
    }
}

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Atom::True => write!(f, "true"),
            Atom::False => write!(f, "false"),
            Atom::SubBexp(bexp) => write!(f, "({bexp})"),
        }
    }
}
