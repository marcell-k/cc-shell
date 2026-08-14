use crate::get_var;
use std::mem;

#[derive(Debug, PartialEq)]
pub enum RedirectMode {
    Truncate,
    Append,
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Word(String),
    Redirect(u8, RedirectMode), // > 0-stdin, 1-stdout, 2-stderr
    Pipeline,                   // |
    Background,                 // &
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
                let fd = if in_token && (buf == "1" || buf == "2") {
                    buf.parse::<u8>().ok()
                } else {
                    None
                };

                let mode = if chars.peek() == Some(&'>') {
                    chars.next();
                    RedirectMode::Append
                } else {
                    RedirectMode::Truncate
                };

                if let Some(fd) = fd {
                    buf.clear();
                    in_token = false;
                    out.push(Token::Redirect(fd, mode));
                } else {
                    if in_token {
                        out.push(Token::Word(mem::take(&mut buf)));
                    }
                    in_token = false;
                    out.push(Token::Redirect(1, mode)); // default
                }
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

            '$' if quote != QuoteState::Single => {
                let mut name = String::new();
                if chars.peek() == Some(&'{') {
                    chars.next();
                    while let Some(&c) = chars.peek() {
                        if c == '}' {
                            chars.next();
                            break;
                        }
                        name.push(c);
                        chars.next();
                    }
                } else if let Some(&c) = chars.peek()
                    && (c.is_ascii_alphabetic() || c == '_')
                {
                    name.push(c);
                    chars.next();
                    while let Some(&c) = chars.peek() {
                        if c.is_ascii_alphanumeric() || c == '_' {
                            name.push(c);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                }
                if name.is_empty() {
                    buf.push('$');
                    in_token = true;
                } else if let Some(val) = get_var(&name)
                    && !val.is_empty()
                {
                    buf.push_str(&val);
                    in_token = true;
                }
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
            '&' if quote == QuoteState::None => {
                if in_token {
                    out.push(Token::Word(mem::take(&mut buf)));
                    in_token = false;
                }
                out.push(Token::Background);
            }
            '|' if quote == QuoteState::None => {
                if in_token {
                    out.push(Token::Word(mem::take(&mut buf)));
                    in_token = false;
                }
                out.push(Token::Pipeline);
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
