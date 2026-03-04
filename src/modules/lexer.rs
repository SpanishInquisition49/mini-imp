use logos::{Lexer, Logos, Skip};
use std::num::ParseIntError;

#[derive(Default, Debug, Clone, PartialEq)]
pub enum LexingError {
    InvalidInteger(String),
    NonAsciiCharacter(char),
    #[default]
    Other,
}

/// Error type returned by calling `lex.slice().parse()` to u8.
impl From<ParseIntError> for LexingError {
    fn from(err: ParseIntError) -> Self {
        use std::num::IntErrorKind::*;
        match err.kind() {
            PosOverflow | NegOverflow => LexingError::InvalidInteger("overflow error".to_owned()),
            _ => LexingError::InvalidInteger("other error".to_owned()),
        }
    }
}

impl LexingError {
    fn from_lexer(lex: &mut logos::Lexer<'_, Token>) -> Self {
        LexingError::NonAsciiCharacter(lex.slice().chars().next().unwrap())
    }
}

fn newline_callback(lex: &mut Lexer<Token>) -> Skip {
    lex.extras.0 += 1;
    lex.extras.1 = lex.span().end;
    Skip
}

#[derive(Logos, Debug, PartialEq)]
#[logos(error(LexingError, LexingError::from_lexer))]
#[logos(extras = (usize, usize))]
#[logos(skip(r"[\n]+", newline_callback))]
#[logos(skip r"[ \t]+")]
pub enum Token {
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("and")]
    And,
    #[token("or")]
    Or,
    #[token("not")]
    Not,
    #[token("<")]
    Less,
    #[token(">")]
    Greater,
    #[token(":")]
    Colon,
    #[token("=")]
    Equal,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token(";")]
    Comma,
    #[token("if")]
    If,
    #[token("then")]
    Then,
    #[token("else")]
    Else,
    #[token("while")]
    While,
    #[token("do")]
    Do,
    #[token("def")]
    Def,
    #[token("main")]
    Main,
    #[token("with")]
    With,
    #[token("input")]
    Input,
    #[token("output")]
    Output,
    #[token("as")]
    As,
    #[regex(r"[a-zA-Z]+", |lex| {String::from(lex.slice())})]
    Word(String),
    #[regex(r"[0-9]+", |lex| lex.slice().parse())]
    Integer(u64),
}
