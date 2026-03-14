use core::fmt;

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
