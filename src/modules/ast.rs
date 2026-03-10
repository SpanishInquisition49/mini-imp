use core::fmt;
use std::collections::HashMap;

pub type Env = HashMap<String, i64>;

#[derive(Debug, Clone)]
pub enum EvalError {
    UnboundVariable(String),
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalError::UnboundVariable(v) => write!(f, "Undefined variable: '{v}'"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Program {
    pub input: String,
    pub output: String,
    pub body: Box<Cmd>,
}

impl Program {
    pub fn eval(&self, input: i64) -> Result<i64, EvalError> {
        let mut env = Env::new();
        env.insert(self.input.clone(), input);
        self.body.eval(&mut env)?;
        env.get(&self.output)
            .copied()
            .ok_or_else(|| EvalError::UnboundVariable(self.output.clone()))
    }
}

#[derive(Debug, Clone)]
pub enum Cmd {
    Block(Box<Cmd>),
    Assign(String, Box<Expr>),
    Seq(Box<Cmd>, Box<Cmd>),
    Ite(Box<BoolExpr>, Box<Cmd>, Box<Cmd>),
    While(Box<BoolExpr>, Box<Cmd>),
    Print(Box<Expr>),
}

impl Cmd {
    pub fn eval(&self, env: &mut Env) -> Result<(), EvalError> {
        match self {
            Cmd::Block(cmd) => cmd.eval(env),
            Cmd::Assign(v, expr) => {
                let val = expr.eval(env)?;
                env.insert(v.clone(), val);
                Ok(())
            }
            Cmd::Seq(c1, c2) => {
                c1.eval(env)?;
                c2.eval(env)
            }
            Cmd::Ite(guard, then, r#else) => {
                if guard.eval(env)? {
                    then.eval(env)
                } else {
                    r#else.eval(env)
                }
            }
            // NOTE: we do not add a max iterations, we could loop forever
            Cmd::While(guard, body) => {
                while guard.eval(env)? {
                    body.eval(env)?
                }
                Ok(())
            }
            Cmd::Print(expr) => {
                println!("{}", expr.eval(env)?);
                Ok(())
            }
        }
    }
}

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

#[derive(Debug, Clone)]
pub enum BoolExpr {
    True,
    False,
    And(Box<BoolExpr>, Box<BoolExpr>),
    Or(Box<BoolExpr>, Box<BoolExpr>),
    Not(Box<BoolExpr>),
    LowerThan(Box<Expr>, Box<Expr>),
    GreaterThan(Box<Expr>, Box<Expr>),
}

impl BoolExpr {
    pub fn eval(&self, env: &Env) -> Result<bool, EvalError> {
        match self {
            BoolExpr::True => Ok(true),
            BoolExpr::False => Ok(false),
            BoolExpr::And(l, r) => Ok(l.eval(env)? && r.eval(env)?),
            BoolExpr::Or(l, r) => Ok(l.eval(env)? || r.eval(env)?),
            BoolExpr::Not(e) => Ok(!e.eval(env)?),
            BoolExpr::LowerThan(l, r) => Ok(l.eval(env)? < r.eval(env)?),
            BoolExpr::GreaterThan(l, r) => Ok(l.eval(env)? > r.eval(env)?),
        }
    }
}
