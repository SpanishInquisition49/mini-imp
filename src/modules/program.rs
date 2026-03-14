use crate::{
    ast::cmd::Cmd,
    modules::eval::{Env, EvalError},
};

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
