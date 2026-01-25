use std::fmt;

use crate::matrix::transform_path;
use crate::parser::{Parser, ParserError};
use crate::simplify::simplify;
use crate::utils::inbox_matrix;
use crate::{BBox, Command, Matrix, Rect};

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

    pub fn commands(&self) -> impl Iterator<Item = &Command> {
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

    /// Split this path into individual subpaths.
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

#[derive(Debug, Clone, PartialEq)]
pub enum CommandF32 {
    Move {
        x: f32,
        y: f32,
    },
    Line {
        x: f32,
        y: f32,
    },
    Cubic {
        x1: f32, // Control point 1
        y1: f32,
        x2: f32, // Control point 2
        y2: f32,
        x: f32, // End point
        y: f32,
    },
    Close,
    Uncovered,
}

impl From<&Command> for CommandF32 {
    fn from(cmd: &Command) -> Self {
        match cmd {
            Command::Move { x, y } => Self::Move {
                x: *x as _,
                y: *y as _,
            },
            Command::Line { x, y } => Self::Line {
                x: *x as _,
                y: *y as _,
            },
            Command::Cubic {
                x1,
                y1,
                x2,
                y2,
                x,
                y,
            } => Self::Cubic {
                x1: *x1 as _,
                y1: *y1 as _,
                x2: *x2 as _,
                y2: *y2 as _,
                x: *x as _,
                y: *y as _,
            },
            Command::Close => Self::Close,
            _ => Self::Uncovered,
        }
    }
}

/// `SimplePath` contains only absolute `M`, `L`, `C`, and `Z`.
#[derive(Debug, Clone)]
pub struct SimplePath {
    commands: Vec<Command>,
    bbox: BBox,
}

impl SimplePath {
    pub fn commands(&self) -> impl Iterator<Item = &Command> {
        self.commands.iter()
    }

    pub fn commands_f32(&self) -> impl Iterator<Item = CommandF32> {
        self.commands.iter().map(|cmd| cmd.into())
    }

    /// Path bounding box
    pub fn bbox(&mut self) -> BBox {
        if self.bbox.min_x == f64::INFINITY {
            self.bbox = crate::bbox::bbox(&self.commands).unwrap();
        }
        self.bbox.clone()
    }

    /// Apply a transformation matrix
    #[must_use]
    pub fn transform(&self, m: &Matrix) -> Self {
        let cmds = transform_path(&self.commands, m);
        Self {
            commands: cmds,
            bbox: BBox::new(),
        }
    }

    /// Fit this path into target rectangle
    #[must_use]
    pub fn fit(&mut self, target: &Rect, keep_aspect_ratio: bool, centered: bool) -> Self {
        let src: Rect = (&self.bbox()).into();
        let m = inbox_matrix(&src, target, keep_aspect_ratio, centered);
        self.transform(&m)
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
