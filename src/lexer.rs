use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Command(char),
    Number(f64),
}

#[derive(Debug, Clone, PartialEq)]
pub enum LexerError {
    UnexpectedCharacter(char),
    InvalidCommand(char),
    InvalidNumber(String),
}

pub(crate) struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(input: &'a str) -> Self {
        Lexer {
            input: input.chars().peekable(),
        }
    }

    fn skip_whitespace_and_commas(&mut self) {
        while let Some(&c) = self.input.peek() {
            if c.is_whitespace() || c == ',' {
                self.input.next();
            } else {
                break;
            }
        }
    }

    fn read_number(&mut self) -> Result<Token, LexerError> {
        let mut num_str = String::new();
        let mut has_decimal = false;
        let mut has_exponent = false;

        // We use a loop and peek to decide exactly when to stop
        while let Some(&c) = self.input.peek() {
            match c {
                // A sign is only part of THIS number if it's the first char
                // OR if it's immediately after an 'e' (scientific notation)
                '-' | '+' => {
                    if num_str.is_empty() || num_str.ends_with('e') || num_str.ends_with('E') {
                        num_str.push(self.input.next().unwrap());
                    } else {
                        // It's a sign for the NEXT number, stop here
                        break;
                    }
                }
                '0'..='9' => {
                    num_str.push(self.input.next().unwrap());
                }
                '.' if !has_decimal && !has_exponent => {
                    has_decimal = true;
                    num_str.push(self.input.next().unwrap());
                }
                'e' | 'E' if !has_exponent => {
                    has_exponent = true;
                    num_str.push(self.input.next().unwrap());
                }
                _ => break, // Any other char (comma, space, letter) stops the number
            }
        }

        num_str
            .parse::<f64>()
            .map(Token::Number)
            .map_err(|_| LexerError::InvalidNumber(num_str))
    }

    fn is_valid_command(c: char) -> bool {
        matches!(
            c.to_ascii_lowercase(),
            'm' | 'l' | 'h' | 'v' | 'c' | 's' | 'q' | 't' | 'a' | 'z'
        )
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token, LexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace_and_commas();

        // Peek to see what's next
        let c = *self.input.peek()?;

        // It's an alphabetic character
        if c.is_ascii_alphabetic() {
            self.input.next(); // Consume it
            if Self::is_valid_command(c) {
                return Some(Ok(Token::Command(c)));
            } else {
                return Some(Err(LexerError::InvalidCommand(c)));
            }
        }

        // It's a number, a sign, or a decimal point
        if c.is_ascii_digit() || c == '-' || c == '+' || c == '.' {
            return Some(self.read_number());
        }

        // It's a character that shouldn't be here (e.g. #, $, %)
        let unknown = self.input.next().unwrap();
        Some(Err(LexerError::UnexpectedCharacter(unknown)))
    }
}

#[cfg(test)]
mod t {
    use super::Token::*;
    use super::*;

    #[test]
    fn tokenize() {
        let test_data = [
            ("", vec![]),
            ("  \n \t", vec![]),
            ("M 10 -5", vec![Command('M'), Number(10.0), Number(-5.0)]),
            ("M10-5", vec![Command('M'), Number(10.0), Number(-5.0)]),
            ("M-10-7", vec![Command('M'), Number(-10.0), Number(-7.0)]),
            (
                "M 0,0 L 10,8 h 1e-4 v 1.5e3 z",
                vec![
                    Command('M'),
                    Number(0.0),
                    Number(0.0),
                    Command('L'),
                    Number(10.0),
                    Number(8.0),
                    Command('h'),
                    Number(0.0001),
                    Command('v'),
                    Number(1500.0),
                    Command('z'),
                ],
            ),
        ];

        for (s, expected) in test_data {
            let lx = Lexer::new(s);
            let ls: Result<Vec<_>, _> = lx.collect();
            assert!(ls.is_ok());
            assert_eq!(ls.unwrap(), expected);
        }
    }

    #[test]
    fn invalid() {
        let invalids = ["M 8 7 X 7 8"];
        for s in invalids {
            let lx = Lexer::new(s);
            let ls: Result<Vec<_>, _> = lx.collect();
            assert!(ls.is_err());
        }
    }
}
