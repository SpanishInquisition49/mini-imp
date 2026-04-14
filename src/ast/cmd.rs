use chumsky::span::SimpleSpan;

use crate::{
    ast::{boolean_exp::BoolExpr, expr::Expr},
    modules::eval::{Env, EvalError},
};

#[derive(Debug, Clone)]
pub enum Cmd {
    Seq(Box<AtomCmd>, Box<Cmd>),
    AtomCmd(Box<AtomCmd>),
}

impl Cmd {
    pub fn eval(&self, env: &mut Env) -> Result<(), EvalError> {
        match self {
            Cmd::Seq(c1, c2) => {
                c1.eval(env)?;
                c2.eval(env)
            }
            Cmd::AtomCmd(cmd) => Ok(cmd.eval(env)?),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AtomCmd {
    Block(Box<Cmd>),
    Assign(String, Box<Expr>, SimpleSpan),
    Ite(Box<BoolExpr>, Box<AtomCmd>, Box<AtomCmd>),
    While(Box<BoolExpr>, Box<AtomCmd>),
    Print(Box<Expr>),
    Skip,
}

impl AtomCmd {
    pub fn eval(&self, env: &mut Env) -> Result<(), EvalError> {
        match self {
            AtomCmd::Assign(v, expr, _) => {
                let val = expr.eval(env)?;
                env.insert(v.clone(), val);
                Ok(())
            }
            AtomCmd::Ite(guard, then, r#else) => {
                if guard.eval(env)? {
                    then.eval(env)
                } else {
                    r#else.eval(env)
                }
            }
            AtomCmd::While(guard, body) => {
                while guard.eval(env)? {
                    body.eval(env)?
                }
                Ok(())
            }
            AtomCmd::Print(expr) => {
                println!("{}", expr.eval(env)?);
                Ok(())
            }
            AtomCmd::Skip => Ok(()),
            AtomCmd::Block(cmd) => Ok(cmd.eval(env)?),
        }
    }
}
