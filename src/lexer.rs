use std::{iter::from_fn, str::Chars};

use anyhow::anyhow;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    LParen,
    RParen,
    LBrace,
    RBrace,

    Comma,
    Dot,
    Semicolon,
    Colon,
    ColonColon,

    Minus,
    Plus,
    Star,
    Slash,

    Ignore,

    Eq,
    EqEq,
    Bang,
    NEq,
    Gt,
    GtEq,
    Lt,
    LtEq,
    Pipe,
    Or,
    Ampersand,
    And,

    Iden(String),
    Str(String),
    Int(i64),
    Float(f64),

    If,
    Else,
    True,
    False,
    Fn,
    For,
    In,
    Null,
    Print,
    Return,
    Let,
    Loop,
}

fn find_matching_block_comment(chars: &mut Chars) -> Option<()> {
    let mut open = 1;
    while let Some(ch) = chars.next() {
        if ch == '/' && chars.next()? == '*' {
            open += 1;
        } else if ch == '*' && chars.next()? == '/' {
            open -= 1;
        }
        if open == 0 {
            return Some(());
        }
    }
    None
}

pub fn lex(source: &str) -> impl Iterator<Item = anyhow::Result<Token>> + '_ {
    use Token::*;

    let mut chars = source.chars();

    from_fn(move || {
        chars = chars.as_str().trim_start().chars();

        Some(match chars.next() {
            None => return None,
            Some(char) => Ok(match char {
                '(' => RParen,
                ')' => LParen,
                '{' => RBrace,
                '}' => LBrace,
                ',' => Comma,
                '.' => Dot,
                ';' => Semicolon,
                '-' => Minus,
                '+' => Plus,
                '*' => Star,
                '/' => match chars.clone().next() {
                    Some('/') => {
                        let _ = chars.by_ref().skip_while(|ch| *ch != '\n');
                        Ignore
                    }
                    Some('*') => {
                        if find_matching_block_comment(&mut chars).is_none() {
                            return Some(Err(anyhow!("unterminated block comment")));
                        }
                        Ignore
                    }
                    _ => Slash,
                },
                '=' => match chars.clone().next() {
                    Some('=') => {
                        chars.next();
                        EqEq
                    }
                    _ => Eq,
                },
                '!' => match chars.clone().next() {
                    Some('=') => {
                        chars.next();
                        NEq
                    }
                    _ => Bang,
                },
                '>' => match chars.clone().next() {
                    Some('=') => {
                        chars.next();
                        GtEq
                    }
                    _ => Gt,
                },
                '<' => match chars.clone().next() {
                    Some('=') => {
                        chars.next();
                        LtEq
                    }
                    _ => Lt,
                },
                '|' => match chars.clone().next() {
                    Some('|') => {
                        chars.next();
                        Or
                    }
                    _ => Pipe,
                },
                '&' => match chars.clone().next() {
                    Some('&') => {
                        chars.next();
                        And
                    }
                    _ => Ampersand,
                },
                ':' => match chars.clone().next() {
                    Some(':') => {
                        chars.next();
                        ColonColon
                    }
                    _ => Colon,
                },
                '"' => {
                    let rest = chars.as_str();
                    let lit = rest.split('"').next().unwrap();
                    let rest = &rest[lit.len()..];
                    chars = rest.chars();
                    if chars.next().is_none() {
                        return Some(Err(anyhow!("Unterminated string literal")));
                    }
                    Str(lit.to_string())
                }

                first_char => {
                    // I fucking hate this so much
                    let lit = if chars.clone().next().unwrap_or(' ').is_whitespace() {
                        ""
                    } else {
                        chars.as_str().split_whitespace().next().unwrap_or("")
                    };

                    chars.nth(lit.chars().count());

                    let lit_string = format!("{first_char}{lit}"); // fuck this allocation
                    match lit_string.as_str() {
                        "if" => If,
                        "else" => Else,
                        "true" => True,
                        "false" => False,
                        "fn" => Fn,
                        "for" => For,
                        "in" => In,
                        "null" => Null,
                        "print" => Print,
                        "return" => Return,
                        "let" => Let,
                        "loop" => Loop,
                        lit if lit
                            .chars()
                            .next()
                            .expect("expected token lenght to be at least 1")
                            .is_numeric() =>
                        {
                            if lit.contains('.') {
                                match lit.parse::<_>() {
                                    Ok(f) => return Some(Ok(Float(f))),
                                    Err(e) => {
                                        return Some(Err(anyhow!(
                                            "Error while parsing float {lit}: {e}"
                                        )))
                                    }
                                };
                            }
                            match lit.parse::<_>() {
                                Ok(f) => return Some(Ok(Int(f))),
                                Err(e) => {
                                    return Some(Err(anyhow!("Error while parsing int {lit}: {e}")))
                                }
                            };
                        }
                        _ if first_char.is_alphabetic() => Iden(lit_string),
                        _ => return Some(Err(anyhow!("unexpected token: {lit_string}"))),
                    }
                }
            }),
        })
    })
}

#[cfg(test)]
mod test {
    use super::lex;
    use super::Token::*;

    #[test]
    fn lex_unisymbols() {
        let test_str = "{}(),.;-+*";
        let expected = vec![
            RBrace, LBrace, RParen, LParen, Comma, Dot, Semicolon, Minus, Plus, Star,
        ];

        for (result, expected) in lex(test_str).map(Result::unwrap).zip(expected.into_iter()) {
            assert_eq!(result, expected, "Token did not match expected");
        }
    }

    #[test]
    fn lex_multi_character_tokens() {
        let test_str = "!= == <= >= && || ::";
        let expected = vec![NEq, EqEq, LtEq, GtEq, And, Or, ColonColon];

        for (result, expected) in lex(test_str).map(Result::unwrap).zip(expected.into_iter()) {
            assert_eq!(result, expected, "Token did not match expected");
        }
    }

    #[test]
    fn lex_literals() {
        let test_str = "\"string\" identifier 1234 56.78 \"\" ";
        let expected = vec![
            Str("string".to_owned()),
            Iden("identifier".to_owned()),
            Int(1234),
            Float(56.78),
            Str("".to_owned()),
        ];

        for (result, expected) in lex(test_str).map(Result::unwrap).zip(expected.into_iter()) {
            assert_eq!(result, expected, "Token did not match expected");
        }
    }

    #[test]
    fn lex_keywords() {
        let test_str = "if else true false fn for in null print return let loop";
        let expected = vec![
            If, Else, True, False, Fn, For, In, Null, Print, Return, Let, Loop,
        ];

        for (result, expected) in lex(test_str).map(Result::unwrap).zip(expected.into_iter()) {
            assert_eq!(result, expected, "Token did not match expected");
            println!("{:?} ok", result);
        }
    }

    #[test]
    fn lex_errors() {
        let test_str = "\"unterminated";
        let result: Vec<_> = lex(test_str).collect();

        assert!(
            result[0].is_err(),
            "Expected an error for unterminated string."
        );
    }

    #[test]
    fn lex_errors_while_parsing_float() {
        let test_str = "12.34.56"; // double dots which is an invalid float
        let result: Vec<_> = lex(test_str).collect();

        assert!(result[0].is_err(), "Expected an error while parsing float.");
    }

    #[test]
    fn lex_errors_while_parsing_int() {
        let test_str = "123a"; // character within a number which is an invalid integer
        let result: Vec<_> = lex(test_str).collect();

        assert!(
            result[0].is_err(),
            "Expected an error while parsing integer."
        );
    }

    #[test]
    fn lex_line_comments() {
        let test_str = "// comment";
        let expected = vec![];

        for (result, expected) in lex(test_str).map(Result::unwrap).zip(expected.into_iter()) {
            assert_eq!(
                result, expected,
                "Token did not match expected for line comment"
            );
        }
    }

    #[test]
    fn lex_block_comments() {
        let test_str = "/*/**/";
        let expected = vec![];

        for (result, expected) in lex(test_str).map(Result::unwrap).zip(expected.into_iter()) {
            assert_eq!(
                result, expected,
                "Token did not match expected for block comment"
            );
        }
    }

    #[test]
    fn lex_newlines() {
        let test_str = "\n\n";
        let expected = vec![];

        for (result, expected) in lex(test_str).map(Result::unwrap).zip(expected.into_iter()) {
            assert_eq!(
                result, expected,
                "Token did not match expected for newlines"
            );
        }
    }
    #[test]
    fn unicode_valid() {
        let test_str = "í ñ";
        let expected = vec![Iden("í".to_owned()), Iden("ñ".to_owned())];

        for (result, expected) in lex(test_str).map(Result::unwrap).zip(expected.into_iter()) {
            assert_eq!(
                result, expected,
                "Token did not match expected for newlines"
            );
        }
    }
    #[test]
    fn invalid_iden() {
        let test_str = "#";
        println!("{:?}", lex(test_str).collect::<Vec<_>>());
        lex(test_str)
            .next()
            .unwrap()
            .expect_err("# shouldn't be a valid identifier");
    }
}
