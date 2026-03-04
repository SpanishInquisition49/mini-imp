use core::fmt;
use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq)]
pub enum Token {
    Error,
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
    LowerThan,
    #[token(">")]
    GreaterThan,
    #[token(":=")]
    Assign,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token(";")]
    SemiColon,
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
    #[regex(r"[a-zA-Z][a-zA-Z0-9]*", |lex| lex.slice().to_string())]
    Identifier(String),
    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    Integer(i64),
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Whitespace,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Error => write!(f, "<Error>"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::And => write!(f, "and"),
            Token::Or => write!(f, "or"),
            Token::Not => write!(f, "not"),
            Token::LowerThan => write!(f, "<"),
            Token::GreaterThan => write!(f, ">"),
            Token::Assign => write!(f, ":="),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::SemiColon => write!(f, ";"),
            Token::If => write!(f, "if"),
            Token::Then => write!(f, "then"),
            Token::Else => write!(f, "else"),
            Token::While => write!(f, "while"),
            Token::Do => write!(f, "do"),
            Token::Def => write!(f, "def"),
            Token::Main => write!(f, "main"),
            Token::With => write!(f, "with"),
            Token::Input => write!(f, "input"),
            Token::Output => write!(f, "output"),
            Token::As => write!(f, "as"),
            Token::Identifier(w) => write!(f, "{w}"),
            Token::Integer(i) => write!(f, "{i}"),
            Token::Whitespace => write!(f, "<Whitespace>"),
        }
    }
}
