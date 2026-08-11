use std::mem;

#[derive(Debug, PartialEq)]
pub enum Token {
    Word(String),
    Redirect, // >
    Pipe,
}

#[derive(PartialEq)]
pub enum QuoteState {
    None,
    Single,
    Double,
}
pub fn tokenize(input: &str) -> Vec<Token> {
    let mut out: Vec<Token> = Vec::new();
    let mut buf = String::new();
    let mut quote = QuoteState::None;
    let mut in_token = false;

    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '>' if quote == QuoteState::None => {
                if in_token {
                    out.push(Token::Word(mem::take(&mut buf)));
                }
                in_token = false;
                out.push(Token::Redirect);
            }
            '\\' if quote == QuoteState::None => {
                if let Some(next) = chars.next() {
                    buf.push(next);
                }
                in_token = true;
            }
            '\\' if quote == QuoteState::Double => {
                match chars.peek() {
                    Some('"') | Some('\\') | Some('$') | Some('`') | Some('\n') => {
                        buf.push(chars.next().unwrap());
                    }
                    _ => {
                        buf.push('\\');
                    }
                }
                in_token = true;
            }

            '\'' if quote != QuoteState::Double => {
                quote = if quote == QuoteState::Single {
                    QuoteState::None
                } else {
                    QuoteState::Single
                };
                in_token = true;
            }

            '\"' if quote != QuoteState::Single => {
                quote = if quote == QuoteState::Double {
                    QuoteState::None
                } else {
                    QuoteState::Double
                };
                in_token = true;
            }
            ' ' | '\t' if quote == QuoteState::None => {
                if in_token {
                    out.push(Token::Word(mem::take(&mut buf)));
                    in_token = false;
                }
            }
            _ => {
                buf.push(c);
                in_token = true;
            }
        }
    }
    if in_token {
        out.push(Token::Word(buf));
    }
    out
}

mod tests {
    use crate::{tokenize, tokenizer::Token};

    #[test]
    fn test_tokenizer() {
        assert_eq!(tokenize("echo '>'"), vec![Token::Word("cho".to_string())])
    }
}
