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
