use chumsky::{input::ValueInput, prelude::*};

use crate::modules::{
    ast::{BoolExpr, Cmd, Expr, Factor, Program, Term},
    lexer::Token,
};

pub fn parser<'a, I>() -> impl Parser<'a, I, Program, extra::Err<Rich<'a, Token>>>
where
    I: ValueInput<'a, Token = Token, Span = SimpleSpan>,
{
    let var = select! { Token::Identifier(s) => s };
    let int_lit = select! { Token::Integer(n) => n };

    let expr = recursive(|expr| {
        let factor = choice((
            var.map(Factor::Var),
            int_lit.map(Factor::Int),
            expr.clone()
                .delimited_by(just(Token::LParen), just(Token::RParen))
                .map(|e| Factor::SubExp(Box::new(e))),
        ));

        let term = factor.clone().map(|f| Term::Fac(Box::new(f))).foldl(
            just(Token::Star).ignore_then(factor.clone()).repeated(),
            |t, f| Term::Mul(Box::new(t), Box::new(f)),
        );

        term.clone().map(|t| Expr::Term(Box::new(t))).foldl(
            choice((
                just(Token::Plus)
                    .ignore_then(term.clone())
                    .map(|t| (true, t)),
                just(Token::Minus)
                    .ignore_then(term.clone())
                    .map(|t| (false, t)),
            ))
            .repeated(),
            |e, (is_add, t)| {
                if is_add {
                    Expr::Add(Box::new(e), Box::new(t))
                } else {
                    Expr::Sub(Box::new(e), Box::new(t))
                }
            },
        )
    });

    let bool_expr = {
        let atom = choice((
            just(Token::True).to(BoolExpr::True),
            just(Token::False).to(BoolExpr::False),
        ));
        let e = expr.clone();
        choice((
            atom.clone(),
            atom.clone()
                .then_ignore(just(Token::And))
                .then(atom.clone())
                .map(|(l, r)| BoolExpr::And(Box::new(l), Box::new(r))),
            atom.clone()
                .then_ignore(just(Token::Or))
                .then(atom.clone())
                .map(|(l, r)| BoolExpr::Or(Box::new(l), Box::new(r))),
            just(Token::Not)
                .ignore_then(atom.clone())
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

        let skip_cmd = just(Token::Skip).to(Cmd::Skip);

        choice((block, if_cmd, while_cmd, assign, print_cmd, skip_cmd))
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
        .ignore_then(var)
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
