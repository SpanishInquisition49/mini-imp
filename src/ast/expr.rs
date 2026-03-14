use core::fmt;

use crate::modules::eval::{Env, EvalError};

#[derive(Debug, Clone)]
pub enum Expr {
    Add(Box<Expr>, Box<Term>),
    Sub(Box<Expr>, Box<Term>),
    Term(Box<Term>),
}

impl Expr {
    pub fn eval(&self, env: &Env) -> Result<i64, EvalError> {
        match self {
            Expr::Add(l, r) => Ok(l.eval(env)? + r.eval(env)?),
            Expr::Sub(l, r) => Ok(l.eval(env)? - r.eval(env)?),
            Expr::Term(term) => Ok(term.eval(env)?),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Add(l, r) => write!(f, "{l} + {r}"),
            Expr::Sub(l, r) => write!(f, "{l} - {r}"),
            Expr::Term(term) => write!(f, "{term}"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Term {
    Mul(Box<Term>, Box<Factor>),
    Fac(Box<Factor>),
}

impl Term {
    pub fn eval(&self, env: &Env) -> Result<i64, EvalError> {
        match self {
            Term::Mul(l, r) => Ok(l.eval(env)? * r.eval(env)?),
            Term::Fac(factor) => Ok(factor.eval(env)?),
        }
    }
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Term::Mul(l, r) => write!(f, "{l} * {r}"),
            Term::Fac(factor) => write!(f, "{factor}"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Factor {
    Var(String),
    Int(i64),
    SubExp(Box<Expr>),
}

impl Factor {
    pub fn eval(&self, env: &Env) -> Result<i64, EvalError> {
        match self {
            Factor::Var(v) => env
                .get(v)
                .copied()
                .ok_or_else(|| EvalError::UnboundVariable(v.clone())),
            Factor::Int(i) => Ok(*i),
            Factor::SubExp(expr) => Ok(expr.eval(env)?),
        }
    }
}

impl fmt::Display for Factor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Factor::Var(var) => write!(f, "{var}"),
            Factor::Int(i) => write!(f, "{i}"),
            Factor::SubExp(expr) => write!(f, "{expr}"),
        }
    }
}
