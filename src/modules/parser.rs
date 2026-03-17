use chumsky::{input::ValueInput, prelude::*};

use crate::{
    ast::{
        boolean_exp::{Atom, BoolExpr},
        cmd::{AtomCmd, Cmd},
        expr::{Expr, Factor, Term},
    },
    modules::{lexer::Token, program::Program},
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

    let bexp = recursive(|bexp| {
        let atom = choice((
            just(Token::True).to(Atom::True),
            just(Token::False).to(Atom::False),
            bexp.clone()
                .delimited_by(just(Token::LParen), just(Token::RParen))
                .map(|b| Atom::SubBexp(Box::new(b))),
        ));

        let base = choice((
            just(Token::Not)
                .ignore_then(bexp.clone())
                .map(|b| BoolExpr::Not(Box::new(b))),
            expr.clone()
                .then_ignore(just(Token::LowerThan))
                .then(expr.clone())
                .map(|(l, r)| BoolExpr::LowerThan(Box::new(l), Box::new(r))),
            expr.clone()
                .then_ignore(just(Token::GreaterThan))
                .then(expr.clone())
                .map(|(l, r)| BoolExpr::GreaterThan(Box::new(l), Box::new(r))),
            atom.clone().map(|a| BoolExpr::Atom(Box::new(a))),
        ));

        base.foldl(
            choice((
                just(Token::And)
                    .ignore_then(atom.clone())
                    .map(|a| (true, a)),
                just(Token::Or)
                    .ignore_then(atom.clone())
                    .map(|a| (false, a)),
            ))
            .repeated(),
            |l, (is_and, r)| {
                if is_and {
                    BoolExpr::And(Box::new(l), Box::new(r))
                } else {
                    BoolExpr::Or(Box::new(l), Box::new(r))
                }
            },
        )
    });

    let cmd = recursive(|cmd| {
        let atom = recursive(|atom| {
            let block = cmd
                .clone()
                .delimited_by(just(Token::LParen), just(Token::RParen))
                .map(|c| AtomCmd::Block(Box::new(c)));

            let assign = var
                .clone()
                .then_ignore(just(Token::Assign))
                .then(expr.clone())
                .map(|(v, e)| AtomCmd::Assign(v, Box::new(e)));

            let if_cmd = just(Token::If)
                .ignore_then(bexp.clone())
                .then_ignore(just(Token::Then))
                .then(atom.clone())
                .then_ignore(just(Token::Else))
                .then(atom.clone())
                .map(|((b, t), e)| AtomCmd::Ite(Box::new(b), Box::new(t), Box::new(e)));

            let while_cmd = just(Token::While)
                .ignore_then(bexp.clone())
                .then_ignore(just(Token::Do))
                .then(atom) // ← consuma atom, non clone
                .map(|(b, c)| AtomCmd::While(Box::new(b), Box::new(c)));

            let print_cmd = just(Token::Print)
                .ignore_then(expr.clone())
                .map(|e| AtomCmd::Print(Box::new(e)));

            let skip_cmd = just(Token::Skip).to(AtomCmd::Skip);

            choice((block, if_cmd, while_cmd, assign, print_cmd, skip_cmd))
        });

        atom.then(just(Token::SemiColon).ignore_then(cmd.clone()).or_not())
            .map(|(c, rest)| match rest {
                Some(r) => Cmd::Seq(Box::new(c), Box::new(r)),
                None => Cmd::AtomCmd(Box::new(c)),
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
