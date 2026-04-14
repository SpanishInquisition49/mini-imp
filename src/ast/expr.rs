use core::fmt;
use std::collections::{HashMap, HashSet};

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

    pub fn extract_const(&self) -> Option<i64> {
        match self {
            Expr::Term(term) => term.extract_const(),
            _ => None,
        }
    }

    pub fn propagate_const(&self, constant_map: &HashMap<String, i64>) -> Option<Expr> {
        match self {
            Expr::Add(left, right) => match (
                left.propagate_const(constant_map),
                right.propagate_const(constant_map),
            ) {
                (None, None) => None,
                (None, Some(right_p)) => Some(Expr::Add(left.clone(), Box::new(right_p))),
                (Some(left_p), None) => Some(Expr::Add(Box::new(left_p), right.clone())),
                (Some(left_p), Some(right_p)) => {
                    Some(Expr::Add(Box::new(left_p), Box::new(right_p)))
                }
            },
            Expr::Sub(left, right) => match (
                left.propagate_const(constant_map),
                right.propagate_const(constant_map),
            ) {
                (None, None) => None,
                (None, Some(right_p)) => Some(Expr::Sub(left.clone(), Box::new(right_p))),
                (Some(left_p), None) => Some(Expr::Sub(Box::new(left_p), right.clone())),
                (Some(left_p), Some(right_p)) => {
                    Some(Expr::Sub(Box::new(left_p), Box::new(right_p)))
                }
            },

            Expr::Term(term) => term
                .propagate_const(constant_map)
                .map(|term_p| Expr::Term(Box::new(term_p))),
        }
    }

    pub fn fold(&self) -> (Expr, bool) {
        match self {
            Expr::Add(left, right) => {
                let (left_exp, left_change) = left.fold();
                let (right_term, right_change) = right.fold();

                if right_term.is_zero() {
                    return (left_exp, true);
                }

                if left_exp.is_zero() {
                    return (Expr::Term(Box::new(right_term)), true);
                }

                if let Some(result) = Self::try_fold_add(&left_exp, &right_term) {
                    return (
                        Expr::Term(Box::new(Term::Fac(Box::new(Factor::Int(result))))),
                        true,
                    );
                }

                (
                    Expr::Add(Box::new(left_exp), Box::new(right_term)),
                    left_change || right_change,
                )
            }
            Expr::Sub(left, right) => {
                let (left_exp, left_change) = left.fold();
                let (right_term, right_change) = right.fold();

                if right_term.is_zero() {
                    return (left_exp, true);
                }

                if let Some(result) = Self::try_fold_sub(&left_exp, &right_term) {
                    return (
                        Expr::Term(Box::new(Term::Fac(Box::new(Factor::Int(result))))),
                        true,
                    );
                }

                (
                    Expr::Sub(Box::new(left_exp), Box::new(right_term)),
                    left_change || right_change,
                )
            }
            Expr::Term(term) => {
                let (term_f, term_c) = term.fold();

                (Expr::Term(Box::new(term_f)), term_c)
            }
        }
    }

    fn is_zero(&self) -> bool {
        if let Expr::Term(t) = self {
            t.is_zero()
        } else {
            false
        }
    }

    fn try_fold_add(left: &Expr, right: &Term) -> Option<i64> {
        if let (Expr::Term(l_term), Term::Fac(r_fac)) = (left, right)
            && let (Term::Fac(l_fac), Factor::Int(r)) = (&**l_term, &**r_fac)
            && let Factor::Int(l) = &**l_fac
        {
            Some(l + r)
        } else {
            None
        }
    }

    fn try_fold_sub(left: &Expr, right: &Term) -> Option<i64> {
        if let (Expr::Term(l_term), Term::Fac(r_fac)) = (left, right)
            && let (Term::Fac(l_fac), Factor::Int(r)) = (&**l_term, &**r_fac)
            && let Factor::Int(l) = &**l_fac
        {
            Some(l - r)
        } else {
            None
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

    pub fn extract_const(&self) -> Option<i64> {
        match self {
            Term::Fac(factor) => factor.extract_const(),
            _ => None,
        }
    }

    pub fn propagate_const(&self, constant_map: &HashMap<String, i64>) -> Option<Term> {
        match self {
            Term::Mul(left, right) => match (
                left.propagate_const(constant_map),
                right.propagate_const(constant_map),
            ) {
                (None, None) => None,
                (None, Some(right_p)) => Some(Term::Mul(left.clone(), Box::new(right_p))),
                (Some(left_p), None) => Some(Term::Mul(Box::new(left_p), right.clone())),
                (Some(left_p), Some(right_p)) => {
                    Some(Term::Mul(Box::new(left_p), Box::new(right_p)))
                }
            },
            Term::Fac(factor) => factor
                .propagate_const(constant_map)
                .map(|factor_p| Term::Fac(Box::new(factor_p))),
        }
    }

    pub fn fold(&self) -> (Term, bool) {
        match self {
            Term::Mul(term, factor) => {
                let (term_f, term_c) = term.fold();
                let (fact_f, fact_c) = factor.fold();

                if term_f.is_zero() {
                    return (Term::Fac(Box::new(Factor::Int(0))), true);
                }

                if fact_f.is_zero() {
                    return (Term::Fac(Box::new(Factor::Int(0))), true);
                }

                if term_f.is_one() {
                    return (Term::Fac(Box::new(fact_f)), true);
                }

                if fact_f.is_one() {
                    return (term_f, true);
                }

                if let Some(result) = Self::try_fold_mul(&term_f, &fact_f) {
                    return (Term::Fac(Box::new(Factor::Int(result))), true);
                }

                (
                    Term::Mul(Box::new(term_f), Box::new(fact_f)),
                    term_c || fact_c,
                )
            }
            Term::Fac(factor) => {
                let (f, c) = factor.fold();
                (Term::Fac(Box::new(f)), c)
            }
        }
    }

    pub fn is_zero(&self) -> bool {
        match self {
            Term::Mul(_, _) => false,
            Term::Fac(factor) => factor.is_zero(),
        }
    }

    pub fn is_one(&self) -> bool {
        match self {
            Term::Mul(_, _) => false,
            Term::Fac(factor) => factor.is_one(),
        }
    }

    fn try_fold_mul(left: &Term, right: &Factor) -> Option<i64> {
        if let (Term::Fac(l_fac), Factor::Int(r)) = (left, right)
            && let Factor::Int(l) = &**l_fac
        {
            Some(l * r)
        } else {
            None
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

    pub fn extract_const(&self) -> Option<i64> {
        match self {
            Factor::Int(c) => Some(*c),
            _ => None,
        }
    }

    pub fn propagate_const(&self, constant_map: &HashMap<String, i64>) -> Option<Factor> {
        match self {
            Factor::Var(v) => constant_map.get(v).map(|c| Factor::Int(*c)),
            Factor::Int(_) => None,
            Factor::SubExp(expr) => expr
                .propagate_const(constant_map)
                .map(|expr_p| Factor::SubExp(Box::new(expr_p))),
        }
    }

    pub fn fold(&self) -> (Factor, bool) {
        match self {
            Factor::SubExp(expr) => {
                let (f, c) = expr.fold();

                // Extract (5 + 0) to 5
                if let Expr::Term(t) = &f
                    && let Term::Fac(inner_f) = &**t
                {
                    return (*inner_f.clone(), true);
                }

                (Factor::SubExp(Box::new(f)), c)
            }
            _ => (self.clone(), false),
        }
    }

    pub fn is_zero(&self) -> bool {
        match self {
            Factor::Var(_) => false,
            Factor::Int(f) => *f == 0,
            Factor::SubExp(_) => false,
        }
    }

    pub fn is_one(&self) -> bool {
        match self {
            Factor::Var(_) => false,
            Factor::Int(f) => *f == 1,
            Factor::SubExp(_) => false,
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
