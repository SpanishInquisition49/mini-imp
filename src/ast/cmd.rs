use crate::{
    ast::{boolean_exp::BoolExpr, expr::Expr},
    modules::eval::{Env, EvalError},
};

#[derive(Debug, Clone)]
pub enum Cmd {
    Block(Box<Cmd>),
    Assign(String, Box<Expr>),
    Seq(Box<Cmd>, Box<Cmd>),
    Ite(Box<BoolExpr>, Box<Cmd>, Box<Cmd>),
    While(Box<BoolExpr>, Box<Cmd>),
    Print(Box<Expr>),
    Skip,
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
            Cmd::Skip => Ok(()),
        }
    }
}
