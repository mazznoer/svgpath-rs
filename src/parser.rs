use std::error::Error;
use std::fmt;
use std::iter::Peekable;

use crate::lexer::{Lexer, LexerError, Token};

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Move {
        x: f64,
        y: f64,
    },
    Line {
        x: f64,
        y: f64,
    },
    Horizontal {
        x: f64,
    },
    Vertical {
        y: f64,
    },
    Cubic {
        x1: f64, // Control point 1
        y1: f64,
        x2: f64, // Control point 2
        y2: f64,
        x: f64, // End point
        y: f64,
    },
    Quadratic {
        x1: f64, // Control point
        y1: f64,
        x: f64, // End point
        y: f64,
    },
    SmoothCubic {
        x2: f64, // Control point 2
        y2: f64,
        x: f64, // End point
        y: f64,
    },
    SmoothQuadratic {
        x: f64,
        y: f64,
    },
    Arc {
        rx: f64, // Radii
        ry: f64,
        x_axis_rotation: f64, // Degrees
        large_arc_flag: bool, // 0 or 1
        sweep_flag: bool,     // 0 or 1
        x: f64,               // End point
        y: f64,
    },
    Close,
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::Move { x, y } => {
                write!(f, "M {} {}", format_n(*x), format_n(*y))
            }
            Command::Line { x, y } => {
                write!(f, "L {} {}", format_n(*x), format_n(*y))
            }
            Command::Horizontal { x } => {
                write!(f, "H {}", format_n(*x))
            }
            Command::Vertical { y } => {
                write!(f, "V {}", format_n(*y))
            }
            Command::Cubic {
                x1,
                y1,
                x2,
                y2,
                x,
                y,
            } => {
                write!(
                    f,
                    "C {} {},{} {},{} {}",
                    format_n(*x1),
                    format_n(*y1),
                    format_n(*x2),
                    format_n(*y2),
                    format_n(*x),
                    format_n(*y)
                )
            }
            Command::Quadratic { x1, y1, x, y } => {
                write!(
                    f,
                    "Q {} {},{} {}",
                    format_n(*x1),
                    format_n(*y1),
                    format_n(*x),
                    format_n(*y)
                )
            }
            Command::SmoothCubic { x2, y2, x, y } => {
                write!(
                    f,
                    "S {} {},{} {}",
                    format_n(*x2),
                    format_n(*y2),
                    format_n(*x),
                    format_n(*y)
                )
            }
            Command::SmoothQuadratic { x, y } => {
                write!(f, "T {} {}", format_n(*x), format_n(*y))
            }
            Command::Arc {
                rx,
                ry,
                x_axis_rotation,
                large_arc_flag,
                sweep_flag,
                x,
                y,
            } => {
                write!(
                    f,
                    "A {} {} {} {} {} {} {}",
                    format_n(*rx),
                    format_n(*ry),
                    format_n(*x_axis_rotation),
                    if *large_arc_flag { 1 } else { 0 },
                    if *sweep_flag { 1 } else { 0 },
                    format_n(*x),
                    format_n(*y)
                )
            }
            Command::Close => write!(f, "Z"),
        }
    }
}

fn format_n(n: f64) -> String {
    if n.fract() == 0.0 {
        format!("{:.0}", n)
    } else {
        format!("{:.2}", n)
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

#[derive(Debug, Clone)]
pub enum ParserError {
    LexerErr(LexerError),
    UnexpectedToken(Token),
    MissingArgument {
        cmd: char,
        expected: usize,
        found: usize,
    },
    NoStartingCommand,
    EndOfStream,
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO
        write!(f, "parser error")
    }
}

impl Error for ParserError {}

impl From<LexerError> for ParserError {
    fn from(err: LexerError) -> Self {
        ParserError::LexerErr(err)
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Point {
    pub x: f64,
    pub y: f64,
}

pub(crate) struct Parser<'a> {
    lexer: Peekable<Lexer<'a>>,
    cursor: Point,
    start_point: Point,
    last_control_point: Option<Point>,
}

impl<'a> Parser<'a> {
    pub(crate) fn new(input: &'a str) -> Self {
        Self {
            lexer: Lexer::new(input).peekable(),
            cursor: Point { x: 0.0, y: 0.0 },
            start_point: Point { x: 0.0, y: 0.0 },
            last_control_point: None,
        }
    }

    pub(crate) fn parse(&mut self) -> Result<Vec<Command>, ParserError> {
        if self.lexer.peek().is_none() {
            return Err(ParserError::EndOfStream);
        }

        let mut commands = Vec::new();
        let mut last_cmd_char: Option<char> = None;

        while let Some(token_result) = self.lexer.peek() {
            // Check for leading numbers ("6" or "6 M 0 0")
            if last_cmd_char.is_none()
                && let Ok(Token::Number(_)) = token_result
            {
                return Err(ParserError::NoStartingCommand);
            }

            // Consume the token we just peeked
            let token = self.lexer.next().unwrap()?;

            match token {
                Token::Command(c) => {
                    // Handle "MM" by validating current command logic
                    let mut current_cmd_char = c;
                    commands.push(self.process_command(current_cmd_char)?);

                    // Handle Implicit Commands and repeated letters
                    while let Some(token_result) = self.lexer.peek() {
                        match token_result {
                            Ok(Token::Number(_)) => {
                                if current_cmd_char.eq_ignore_ascii_case(&'M') {
                                    current_cmd_char = if current_cmd_char.is_lowercase() {
                                        'l'
                                    } else {
                                        'L'
                                    };
                                }
                                commands.push(self.process_command(current_cmd_char)?);
                            }
                            // If another command follows immediately (e.g., "MM"),
                            // the outer loop will handle it. We break here.
                            _ => break,
                        }
                    }
                    last_cmd_char = Some(current_cmd_char);
                }
                Token::Number(n) => return Err(ParserError::UnexpectedToken(Token::Number(n))),
            }
        }
        Ok(commands)
    }

    /// Internal logic to consume required numbers for a specific command char
    /// and convert them to absolute coordinates.
    fn process_command(&mut self, c: char) -> Result<Command, ParserError> {
        let is_rel = c.is_lowercase();
        let cmd_type = c.to_ascii_uppercase();

        match cmd_type {
            'M' => {
                let p = self.get_abs_point(is_rel)?;
                self.cursor = p;
                self.start_point = p;
                self.last_control_point = None;
                Ok(Command::Move { x: p.x, y: p.y })
            }
            'L' => {
                let p = self.get_abs_point(is_rel)?;
                self.cursor = p;
                self.last_control_point = None;
                Ok(Command::Line { x: p.x, y: p.y })
            }
            'H' => {
                let mut x = self.next_num()?;
                if is_rel {
                    x += self.cursor.x;
                }
                self.cursor.x = x;
                self.last_control_point = None;
                Ok(Command::Horizontal { x })
            }
            'V' => {
                let mut y = self.next_num()?;
                if is_rel {
                    y += self.cursor.y;
                }
                self.cursor.y = y;
                self.last_control_point = None;
                Ok(Command::Vertical { y })
            }
            'C' => {
                let p1 = self.get_abs_point(is_rel)?;
                let p2 = self.get_abs_point(is_rel)?;
                let p = self.get_abs_point(is_rel)?;
                self.last_control_point = Some(p2); // CP2 is used for next 'S' reflection
                self.cursor = p;
                Ok(Command::Cubic {
                    x1: p1.x,
                    y1: p1.y,
                    x2: p2.x,
                    y2: p2.y,
                    x: p.x,
                    y: p.y,
                })
            }
            'S' => {
                let p2 = self.get_abs_point(is_rel)?;
                let p = self.get_abs_point(is_rel)?;
                self.last_control_point = Some(p2);
                self.cursor = p;
                Ok(Command::SmoothCubic {
                    x2: p2.x,
                    y2: p2.y,
                    x: p.x,
                    y: p.y,
                })
            }
            'Q' => {
                let p1 = self.get_abs_point(is_rel)?;
                let p = self.get_abs_point(is_rel)?;
                self.last_control_point = Some(p1); // CP1 is used for next 'T' reflection
                self.cursor = p;
                Ok(Command::Quadratic {
                    x1: p1.x,
                    y1: p1.y,
                    x: p.x,
                    y: p.y,
                })
            }
            'T' => {
                let _p1 = self.reflect_control_point();
                let p = self.get_abs_point(is_rel)?;
                // The reflected point is technically the control point for this segment
                self.last_control_point = Some(self.reflect_control_point());
                self.cursor = p;
                Ok(Command::SmoothQuadratic { x: p.x, y: p.y })
            }
            'A' => {
                let rx = self.next_num()?;
                let ry = self.next_num()?;
                let rot = self.next_num()?;
                let large = self.next_num()? != 0.0;
                let sweep = self.next_num()? != 0.0;
                let p = self.get_abs_point(is_rel)?;
                self.cursor = p;
                self.last_control_point = None;
                Ok(Command::Arc {
                    rx,
                    ry,
                    x_axis_rotation: rot,
                    large_arc_flag: large,
                    sweep_flag: sweep,
                    x: p.x,
                    y: p.y,
                })
            }
            'Z' => {
                self.cursor = self.start_point;
                self.last_control_point = None;

                // Check if a number follows Z illegally
                if let Some(Ok(Token::Number(n))) = self.lexer.peek() {
                    return Err(ParserError::UnexpectedToken(Token::Number(*n)));
                }
                Ok(Command::Close)
            }
            _ => Err(ParserError::LexerErr(LexerError::InvalidCommand(c))),
        }
    }

    /// Helper to fetch the next two numbers and return an absolute Point
    fn get_abs_point(&mut self, is_rel: bool) -> Result<Point, ParserError> {
        let mut x = self.next_num()?;
        let mut y = self.next_num()?;
        if is_rel {
            x += self.cursor.x;
            y += self.cursor.y;
        }
        Ok(Point { x, y })
    }

    /// Calculates the reflection of the previous control point.
    /// If the previous command was not a curve, it returns the current cursor.
    fn reflect_control_point(&self) -> Point {
        match self.last_control_point {
            Some(last) => Point {
                x: 2.0 * self.cursor.x - last.x,
                y: 2.0 * self.cursor.y - last.y,
            },
            None => self.cursor,
        }
    }

    /// Pulls the next number from the lexer or returns an error
    fn next_num(&mut self) -> Result<f64, ParserError> {
        match self.lexer.next() {
            Some(Ok(Token::Number(n))) => Ok(n),
            Some(Ok(Token::Command(c))) => Err(ParserError::UnexpectedToken(Token::Command(c))),
            Some(Err(e)) => Err(ParserError::LexerErr(e)),
            None => Err(ParserError::EndOfStream),
        }
    }
}

#[cfg(test)]
mod t {
    use super::*;
    use std::fmt::Write;

    fn stringify(commands: &[Command]) -> String {
        let mut s = String::new();
        for cmd in commands {
            write!(&mut s, "{cmd} ").unwrap();
        }
        s.pop();
        s
    }

    #[test]
    fn basic() {
        let test_data = [
            ("M 0 0", "M 0 0"),
            ("M 0 0 H 10 V 8Z", "M 0 0 H 10 V 8 Z"),
            ("M5,7h10v-13z", "M 5 7 H 15 V -6 Z"),
            ("M5,7l-3,3", "M 5 7 L 2 10"),
        ];
        for (s, out) in test_data {
            let mut p = Parser::new(s);
            let res = p.parse();
            assert!(res.is_ok());
            assert_eq!(out, stringify(&res.unwrap()));
        }
    }

    #[test]
    fn invalid() {
        let invalids = [
            "",
            "  ",
            "\n\t ",
            "M",
            "5",
            "MM 0 5 L 6 9",
            //"M 10.5.5",
            "M 0 0 L 1e2e3",
            //"L 7 5",
            "M 10 @ 20",
            "M -.e10",
            "M 9,5 h 20 Z 0",
            "M 5 L 7 9 Z",
            "M 0 L 6 Z",
            "X 10 20",
            "M 10 10 L Infinity NaN",
            "M 10 10 L 20 20 .",
            "M 0,0 foo",
            "M 0.4 -0.4 a 0.7 0.7 0 0.7 0",
            //"M 13,000.56 L 20 20",
        ];
        for s in invalids {
            let mut p = Parser::new(s);
            let res = p.parse();
            if res.is_ok() {
                println!("{s}");
            }
            assert!(res.is_err());
        }
    }
}
