use core::fmt;
use std::collections::HashSet;

use crate::modules::eval::{Env, EvalError};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
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

    pub fn vars(&self) -> HashSet<String> {
        let mut vars = HashSet::new();
        match self {
            Expr::Add(expr, term) => {
                vars.extend(expr.vars());
                vars.extend(term.vars());
            }
            Expr::Sub(expr, term) => {
                vars.extend(expr.vars());
                vars.extend(term.vars());
            }
            Expr::Term(term) => {
                vars.extend(term.vars());
            }
        };
        vars
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

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
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

    pub fn vars(&self) -> HashSet<String> {
        let mut vars = HashSet::new();
        match self {
            Term::Mul(term, factor) => {
                vars.extend(term.vars());
                vars.extend(factor.vars());
                vars
            }
            Term::Fac(factor) => {
                vars.extend(factor.vars());
                vars
            }
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

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
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

    pub fn vars(&self) -> HashSet<String> {
        let mut vars = HashSet::new();
        match self {
            Factor::Var(v) => {
                vars.insert(v.clone());
                vars
            }
            Factor::Int(_) => vars,
            Factor::SubExp(expr) => expr.vars(),
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
