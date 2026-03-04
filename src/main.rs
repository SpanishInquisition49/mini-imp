use logos::Logos;

use crate::modules::lexer::Token;

mod modules;

fn main() {
    let lex = Token::lexer(
        "def main with input in output out as
out := 1;
i := 1;
while i < in + 1 do (
out := out * i;
i = i + 1
)",
    );

    lex.for_each(|t| match t {
        Ok(tok) => println!("{tok:#?}"),
        Err(e) => println!("Error: {e:#?}"),
    });
}
