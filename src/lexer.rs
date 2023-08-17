use std::{iter::from_fn, str::Chars};

use anyhow::anyhow;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Token<'a> {
    LParen,
    RParen,
    LBrace,
    RBrace,

    Comma,
    Dot,
    Semicolon,
    Colon,
    ColonColon,
    NewLine,
    Whitespace,

    Minus,
    Plus,
    Star,
    Slash,
    LineComment,
    BlockComment,

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

    Iden(&'a str),
    Str(&'a str),
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

    EOF,
}

fn peek(n: usize, it: &Chars) -> Option<char> {
    it.clone().nth(n)
}

pub fn lex<'a>(source: &'a str) -> impl Iterator<Item = anyhow::Result<Token<'a>>> {
    use Token::*;

    let mut chars = source.chars();
    let mut ended = false;

    from_fn(move || {
        if ended {
            return None;
        }
        Some(match chars.next() {
            None => {
                ended = true;
                Ok(EOF)
            }
            Some(char) => Ok(match char {
                '\n' => NewLine,
                '\r' => match peek(1, &chars) {
                    Some('\n') => {
                        chars.next();
                        NewLine
                    }
                    Some(_) => Whitespace,
                    None => return Some(Err(anyhow!("unexpected EOF after \\r"))),
                },
                _ if char.is_whitespace() => Whitespace,

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
                '/' => match peek(1, &chars) {
                    Some('/') => {
                        chars.next();
                        LineComment
                    }
                    Some('*') => {
                        chars.next();
                        BlockComment
                    }
                    _ => Slash,
                },
                '=' => match peek(1, &chars) {
                    Some('=') => {
                        chars.next();
                        EqEq
                    }
                    _ => Eq,
                },
                '!' => match peek(1, &chars) {
                    Some('=') => {
                        chars.next();
                        NEq
                    }
                    _ => Bang,
                },
                '>' => match peek(1, &chars) {
                    Some('=') => {
                        chars.next();
                        GtEq
                    }
                    _ => Gt,
                },
                '<' => match peek(1, &chars) {
                    Some('=') => {
                        chars.next();
                        LtEq
                    }
                    _ => Lt,
                },
                '|' => match peek(1, &chars) {
                    Some('|') => {
                        chars.next();
                        Or
                    }
                    _ => Pipe,
                },
                '&' => match peek(1, &chars) {
                    Some('&') => {
                        chars.next();
                        And
                    }
                    _ => Ampersand,
                },
                ':' => match peek(1, &chars) {
                    Some(':') => {
                        chars.next();
                        ColonColon
                    }
                    _ => Colon,
                },
                '"' => match chars.as_str().split('"').next() {
                    Some(lit) => {
                        let lit = &lit[lit.len() + 1..];
                        chars = lit.chars();
                        Str(lit)
                    }
                    None => return Some(Err(anyhow!("Reached EOF while parsing string literal"))),
                },

                _ => match chars.as_str().split_whitespace().next() {
                    None => return Some(Err(anyhow!("Reached EOF while parsing token"))),
                    Some(lit) => {
                        let lit = &lit[lit.len()..];
                        chars = lit.chars();
                        match lit {
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
                                        return Some(Err(anyhow!(
                                            "Error while parsing int {lit}: {e}"
                                        )))
                                    }
                                };
                            }
                            lit => Iden(lit),
                        }
                    }
                },
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
            RBrace, LBrace, RParen, LParen, Comma, Dot, Semicolon, Minus, Plus, Star, EOF,
        ];

        for (result, expected) in lex(test_str).map(Result::unwrap).zip(expected.into_iter()) {
            assert_eq!(result, expected, "Token did not match expected");
        }
    }

    #[test]
    fn lex_multi_character_tokens() {
        let test_str = "!= == <= >= && || ::";
        let expected = vec![
            NEq, Whitespace, EqEq, Whitespace, LtEq, Whitespace, GtEq, Whitespace, And, Whitespace,
            Or, Whitespace, ColonColon, EOF,
        ];

        for (result, expected) in lex(test_str).map(Result::unwrap).zip(expected.into_iter()) {
            assert_eq!(result, expected, "Token did not match expected");
        }
    }

    #[test]
    fn lex_literals() {
        let test_str = "\"string\" identifier 1234 56.78";
        let expected = vec![
            Str("string"),
            Whitespace,
            Iden("identifier"),
            Whitespace,
            Int(1234),
            Whitespace,
            Float(56.78),
            EOF,
        ];

        for (result, expected) in lex(test_str).map(Result::unwrap).zip(expected.into_iter()) {
            assert_eq!(result, expected, "Token did not match expected");
        }
    }

    #[test]
    fn lex_keywords() {
        let test_str = "if else true false fn for in null print return let loop";
        let expected = vec![
            If, Whitespace, Else, Whitespace, True, Whitespace, False, Whitespace, Fn, Whitespace,
            For, Whitespace, In, Whitespace, Null, Whitespace, Print, Whitespace, Return,
            Whitespace, Let, Whitespace, Loop, EOF,
        ];

        for (result, expected) in lex(test_str).map(Result::unwrap).zip(expected.into_iter()) {
            assert_eq!(result, expected, "Token did not match expected");
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
    fn lex_errors_unexpected_eof_after_cr() {
        let test_str = "\r";
        let result: Vec<_> = lex(test_str).collect();

        assert!(
            result[0].is_err(),
            "Expected an error for unexpected EOF after \\r."
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
}
