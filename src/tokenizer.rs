use std::mem;

#[derive(PartialEq)]
pub enum QuoteState {
    None,
    Single,
    Double,
}
pub fn tokenize(input: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut buf = String::new();
    let mut quote = QuoteState::None;
    let mut in_token = false;

    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
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
                    out.push(mem::take(&mut buf));
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
        out.push(buf);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::tokenize;

    #[test]
    fn test_single_quote_tokenize() {
        assert_eq!(tokenize("hello"), vec!["hello"]);
        assert_eq!(tokenize("hello world"), vec!["hello", "world"]);
        assert_eq!(tokenize("hello    world"), vec!["hello", "world"]);
        assert_eq!(tokenize("'hello    world'"), vec!["hello    world"]);
        assert_eq!(tokenize("''"), vec![""]);
        assert_eq!(tokenize("hello''world"), vec!["helloworld"]);
        assert_eq!(tokenize("'hello''world'"), vec!["helloworld"]);
        assert_eq!(
            tokenize("cat '/tmp/file name' '/tmp/file name with spaces'"),
            vec!["cat", "/tmp/file name", "/tmp/file name with spaces"]
        );
    }
    #[test]
    fn test_double_quote_tokenize() {
        assert_eq!(
            tokenize("echo \"hello    world\""),
            vec!["echo", "hello    world"]
        ); // spaces preserved inside double quotes

        assert_eq!(
            tokenize("echo \"hello\"\"world\""),
            vec!["echo", "helloworld"]
        ); // adjacent double-quoted strings concatenate

        assert_eq!(tokenize("echo \"hello\"world"), vec!["echo", "helloworld"]); // quoted + unquoted concatenate

        assert_eq!(
            tokenize("echo \"hello\" \"world\""),
            vec!["echo", "hello", "world"]
        ); // space outside quotes = separate args

        assert_eq!(
            tokenize("echo \"shell's test\""),
            vec!["echo", "shell's test"]
        ); // single quote literal inside double quotes

        assert_eq!(
            tokenize("echo 'hello'\"world\""),
            vec!["echo", "helloworld"]
        ); // single + double quote concat, mixed

        assert_eq!(tokenize("echo \"\""), vec!["echo", ""]); // empty double-quoted arg
    }
}
