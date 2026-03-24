use core::fmt;
use std::collections::HashSet;

use crate::{
    ast::expr::Expr,
    modules::eval::{Env, EvalError},
};

#[derive(Debug, Clone)]
pub enum BoolExpr {
    And(Box<BoolExpr>, Box<Atom>),
    Or(Box<BoolExpr>, Box<Atom>),
    Not(Box<BoolExpr>),
    LowerThan(Box<Expr>, Box<Expr>),
    GreaterThan(Box<Expr>, Box<Expr>),
    Atom(Box<Atom>),
}

impl BoolExpr {
    pub fn eval(&self, env: &Env) -> Result<bool, EvalError> {
        match self {
            BoolExpr::And(l, r) => Ok(l.eval(env)? && r.eval(env)?),
            BoolExpr::Or(l, r) => Ok(l.eval(env)? || r.eval(env)?),
            BoolExpr::Not(e) => Ok(!e.eval(env)?),
            BoolExpr::LowerThan(l, r) => Ok(l.eval(env)? < r.eval(env)?),
            BoolExpr::GreaterThan(l, r) => Ok(l.eval(env)? > r.eval(env)?),
            BoolExpr::Atom(atom) => Ok(atom.eval(env)?),
        }
    }

    pub fn vars(&self) -> HashSet<String> {
        let mut vars = HashSet::new();
        match self {
            BoolExpr::And(bool_expr, atom) => {
                vars.extend(bool_expr.vars());
                vars.extend(atom.vars());
            }
            BoolExpr::Or(bool_expr, atom) => {
                vars.extend(bool_expr.vars());
                vars.extend(atom.vars());
            }
            BoolExpr::Not(bool_expr) => {
                vars.extend(bool_expr.vars());
            }
            BoolExpr::LowerThan(expr, expr1) => {
                vars.extend(expr.vars());
                vars.extend(expr1.vars());
            }
            BoolExpr::GreaterThan(expr, expr1) => {
                vars.extend(expr.vars());
                vars.extend(expr1.vars());
            }
            BoolExpr::Atom(atom) => {
                vars.extend(atom.vars());
            }
        };
        vars
    }
}

impl fmt::Display for BoolExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BoolExpr::And(l, r) => write!(f, "{l} and {r}"),
            BoolExpr::Or(l, r) => write!(f, "{l} or {r}"),
            BoolExpr::Not(bexp) => write!(f, "not {bexp}"),
            BoolExpr::LowerThan(l, r) => write!(f, "{l} < {r}"),
            BoolExpr::GreaterThan(l, r) => write!(f, "{l} > {r}"),
            BoolExpr::Atom(atom) => write!(f, "{atom}"),
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
