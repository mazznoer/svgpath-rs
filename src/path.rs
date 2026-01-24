use std::fmt;

use crate::matrix::transform_path;
use crate::parser::{Parser, ParserError};
use crate::simplify::simplify;
use crate::{BBox, Command, Matrix};

// --- Path

/// `Path` contains only absolute commands.
#[derive(Debug, Clone)]
pub struct Path {
    commands: Vec<Command>,
}

/// Parse SVG Path string, convert all commands into absolute commands.
pub fn parse(s: &str) -> Result<Path, ParserError> {
    let mut p = Parser::new(s);
    let cmds = p.parse()?;
    Ok(Path { commands: cmds })
}

impl Path {
    pub fn new(cmds: &[Command]) -> Self {
        Self {
            commands: cmds.into(),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Command> {
        self.commands.iter()
    }

    /// `H`, `V` --> `L`
    /// `Q`, `S`, `T`, `A` --> `C`
    #[must_use]
    pub fn simplify(&self) -> SimplePath {
        let cmds = simplify(&self.commands);
        SimplePath {
            commands: cmds,
            bbox: BBox::new(),
        }
    }

    /// Split this path into individual non-connected subpaths.
    #[must_use]
    pub fn split(&self) -> Vec<Path> {
        let mut paths = Vec::new();
        let mut current_path = Vec::new();

        for cmd in &self.commands {
            match cmd {
                // A Move command starts a new subpath
                Command::Move { .. } => {
                    if !current_path.is_empty() {
                        paths.push(Path {
                            commands: current_path.clone(),
                        });
                        current_path.clear();
                    }
                    current_path.push(cmd.clone());
                }
                // All other commands belong to the current subpath
                _ => {
                    current_path.push(cmd.clone());
                }
            }
        }

        // Push the final subpath if it exists
        if !current_path.is_empty() {
            paths.push(Path {
                commands: current_path,
            });
        }

        paths
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

// --- SimplePath

/// `SimplePath` contains only absolute `M`, `L`, `C`, and `Z`.
#[derive(Debug, Clone)]
pub struct SimplePath {
    commands: Vec<Command>,
    bbox: BBox,
}

impl SimplePath {
    pub fn iter(&self) -> impl Iterator<Item = &Command> {
        self.commands.iter()
    }

    /// Path bounding box
    pub fn bbox(&mut self) -> BBox {
        if self.bbox.min_x == f64::INFINITY {
            self.bbox = crate::bbox::bbox(&self.commands).unwrap();
        }
        self.bbox.clone()
    }

    #[must_use]
    pub fn transform(&self, m: &Matrix) -> Self {
        let cmds = transform_path(&self.commands, m);
        Self {
            commands: cmds,
            bbox: BBox::new(),
        }
    }

    /// Check if this path consist only of straight lines.
    pub fn is_flat(&self) -> bool {
        for cmd in &self.commands {
            if let Command::Cubic { .. } = cmd {
                return false;
            }
        }
        true
    }
}

impl fmt::Display for SimplePath {
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
