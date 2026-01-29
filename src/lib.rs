//! # svgpath
//!
//! ## Example
//!
//! ```rust
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use svgpath::Command;
//!
//! let s = "
//! M 10,30
//! A 20,20 0,0,1 50,30
//! A 20,20 0,0,1 90,30
//! Q 90,60 50,90
//! Q 10,60 10,30
//! Z";
//!
//! // Parse the SVG path string
//! let p = svgpath::parse(s)?;
//!
//! // Convert to SimplePath
//! let sp = p.simplify();
//!
//! // Scale and translate to fit inside 700 x 700 rectangle at X=50 and Y=50
//! let rect = svgpath::Rect::new(50.0, 50.0, 700.0, 700.0);
//! let sp = sp.fit(&rect, true, true);
//!
//! // Rotate 35 degrees by its center point and scale up
//! let center = sp.bbox().center();
//! let m = svgpath::Matrix::new()
//!     .rotate_by(35.0, center.x, center.y)
//!     .scale(2.0, 2.0);
//! let sp = sp.transform(&m);
//!
//! // Print SVG path d.
//! println!("{sp}");
//!
//! for cmd in sp.commands() {
//!     match cmd {
//!         Command::Move{x, y} => println!("move {x} {y}"),
//!         Command::Line{x, y} => println!("line {x} {y}"),
//!         Command::Cubic{x1, y1, x2, y2, x, y} => println!("cubic {x1} {y1} {x2} {y2} {x} {y}"),
//!         Command::Close => println!("close"),
//!         _ => {},
//!     }
//! }
//!
//! # Ok(())
//! # }
//! ```
//!

mod bbox;
mod lexer;
mod matrix;
mod parser;
mod path;
mod reverse;
mod simplify;
mod utils;

pub use bbox::BBox;
pub use matrix::Matrix;
pub use parser::{Command, ParserError, Point};
pub use path::{CommandF32, Path, SimplePath, parse};
pub use utils::Rect;
