use std::fmt;

use crate::Command;
use crate::parser::{Parser, ParserError};

// --- Path

#[derive(Debug, Clone)]
pub struct Path {
    commands: Vec<Command>,
}

pub fn parse(s: &str) -> Result<Path, ParserError> {
    let mut p = Parser::new(s);
    let cmds = p.parse()?;
    Ok(Path { commands: cmds })
}

impl Path {
    pub fn iter(&self) -> impl Iterator<Item = &Command> {
        self.commands.iter()
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let last = self.commands.len() - 1;
        for (i, cmd) in self.commands.iter().enumerate() {
            if i == last {
                write!(f, "{cmd}")?
            } else {
                write!(f, "{cmd} ")?
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod t {
    use super::*;

    #[test]
    fn basic() {
        let s = "M 7,9 L 100,75 h -50 z";
        let p = parse(s);
        assert!(p.is_ok());
        let p = p.unwrap();

        assert_eq!(p.to_string(), "M 7 9 L 100 75 H 50 Z");

        let mut it = p.iter();
        assert_eq!(it.next().unwrap().to_string(), "M 7 9");
    }

    #[test]
    fn invalid() {
        let test_data = [
            "",
            "  \n \t ",
            "M",
            "M 0,0 L",
            "M 5 5 H 10 X 7 3 Z",
            "M 3 4 5 H 10 Z",
        ];
        for s in test_data {
            let p = parse(s);
            assert!(p.is_err());
        }
    }
}
