use chumsky::{input::ValueInput, prelude::*};

use crate::modules::{
    ast::{BoolExpr, Cmd, Expr, Program},
    lexer::Token,
};

pub fn parser<'a, I>() -> impl Parser<'a, I, Program, extra::Err<Rich<'a, Token>>>
where
    I: ValueInput<'a, Token = Token, Span = SimpleSpan>,
{
    let var = select! { Token::Identifier(s) => s };

    let int_lit = select! { Token::Integer(n) => n };

    let expr = recursive(|expr| {
        let atom = choice((
            var.map(Expr::Var),
            int_lit.map(Expr::Int),
            expr.clone()
                .delimited_by(just(Token::LParen), just(Token::RParen)),
        ));

        let product = atom
            .clone()
            .foldl(just(Token::Star).ignore_then(atom).repeated(), |l, r| {
                Expr::Mul(Box::new(l), Box::new(r))
            });

        product.clone().foldl(
            just(Token::Plus)
                .to(true)
                .or(just(Token::Minus).to(false))
                .then(product)
                .repeated(),
            |l, (add, r)| {
                if add {
                    Expr::Add(Box::new(l), Box::new(r))
                } else {
                    Expr::Sub(Box::new(l), Box::new(r))
                }
            },
        )
    });

    let bool_expr = {
        let e = expr.clone();
        choice((
            just(Token::True).to(BoolExpr::True),
            just(Token::False).to(BoolExpr::False),
            e.clone()
                .then_ignore(just(Token::And))
                .then(e.clone())
                .map(|(l, r)| BoolExpr::And(Box::new(l), Box::new(r))),
            e.clone()
                .then_ignore(just(Token::Or))
                .then(e.clone())
                .map(|(l, r)| BoolExpr::Or(Box::new(l), Box::new(r))),
            just(Token::Not)
                .ignore_then(e.clone())
                .map(|x| BoolExpr::Not(Box::new(x))),
            e.clone()
                .then_ignore(just(Token::LowerThan))
                .then(e.clone())
                .map(|(l, r)| BoolExpr::LowerThan(Box::new(l), Box::new(r))),
            e.clone()
                .then_ignore(just(Token::GreaterThan))
                .then(e.clone())
                .map(|(l, r)| BoolExpr::GreaterThan(Box::new(l), Box::new(r))),
        ))
    };

    let cmd = recursive(|cmd| {
        let block = cmd
            .clone()
            .delimited_by(just(Token::LParen), just(Token::RParen))
            .map(|c| Cmd::Block(Box::new(c)));

        let assign = var
            .clone()
            .then_ignore(just(Token::Assign))
            .then(expr.clone())
            .map(|(v, e)| Cmd::Assign(v, Box::new(e)));

        let if_cmd = just(Token::If)
            .ignore_then(bool_expr.clone())
            .then_ignore(just(Token::Then))
            .then(cmd.clone())
            .then_ignore(just(Token::Else))
            .then(cmd.clone())
            .map(|((b, t), e)| Cmd::Ite(Box::new(b), Box::new(t), Box::new(e)));

        let while_cmd = just(Token::While)
            .ignore_then(bool_expr.clone())
            .then_ignore(just(Token::Do))
            .then(cmd.clone())
            .map(|(b, c)| Cmd::While(Box::new(b), Box::new(c)));

        let print_cmd = just(Token::Print)
            .ignore_then(expr.clone())
            .map(|e| Cmd::Print(Box::new(e)));

        choice((block, if_cmd, while_cmd, assign, print_cmd))
            .then(just(Token::SemiColon).ignore_then(cmd.clone()).or_not())
            .map(|(c, rest)| match rest {
                Some(r) => Cmd::Seq(Box::new(c), Box::new(r)),
                None => c,
            })
    });

    just(Token::Def)
        .ignore_then(just(Token::Main))
        .ignore_then(just(Token::With))
        .ignore_then(just(Token::Input))
        .ignore_then(var.clone())
        .then_ignore(just(Token::Output))
        .then(var)
        .then_ignore(just(Token::As))
        .then(cmd)
        .map(|((inp, out), body)| Program {
            input: inp,
            output: out,
            body: Box::new(body),
        })
}
