use chumsky::span::SimpleSpan;

use crate::modules::eval::{Env, EvalError};

pub type Span = SimpleSpan;

#[derive(Debug, Clone)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

pub trait Eval {
    type Output;
    fn eval(&self, env: &mut Env) -> Result<Self::Output, EvalError>;
}

impl<T: Eval> Eval for Spanned<T> {
    type Output = T::Output;
    fn eval(&self, env: &mut Env) -> Result<Self::Output, EvalError> {
        self.node.eval(env)
    }
}

impl<T: Eval> Eval for Box<T> {
    type Output = T::Output;
    fn eval(&self, env: &mut Env) -> Result<Self::Output, EvalError> {
        self.as_ref().eval(env)
    }
}
