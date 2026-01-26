use crate::{BBox, Command, Matrix};

#[derive(Debug, Clone, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

impl From<&BBox> for Rect {
    fn from(bb: &BBox) -> Self {
        Self {
            x: bb.min_x,
            y: bb.min_y,
            width: bb.width(),
            height: bb.height(),
        }
    }
}

pub(crate) fn inbox_matrix(
    src: &Rect,
    target: &Rect,
    keep_aspect_ratio: bool,
    centered: bool,
) -> Matrix {
    // Prevent division by zero
    if src.width == 0.0 || src.height == 0.0 {
        return Matrix::new().translate(target.x, target.y);
    }

    let mut scale_x = target.width / src.width;
    let mut scale_y = target.height / src.height;

    if keep_aspect_ratio {
        // Use the smaller scale factor to ensure it fits both dimensions
        let uniform_scale = scale_x.min(scale_y);
        scale_x = uniform_scale;
        scale_y = uniform_scale;
    }

    // Calculate translation
    let mut tx = target.x - src.x * scale_x;
    let mut ty = target.y - src.y * scale_y;

    if centered {
        // Add offset to center the scaled rectangle within the target rectangle
        if scale_x < (target.width / src.width) {
            tx += (target.width - src.width * scale_x) / 2.0;
        }
        if scale_y < (target.height / src.height) {
            ty += (target.height - src.height * scale_y) / 2.0;
        }
    }

    Matrix {
        a: scale_x,
        b: 0.0,
        c: 0.0,
        d: scale_y,
        e: tx,
        f: ty,
    }
}

pub(crate) fn split(commands: &[Command]) -> Vec<Vec<Command>> {
    let mut paths = Vec::new();
    let mut current_path = Vec::new();

    for cmd in commands {
        match cmd {
            // A Move command starts a new subpath
            Command::Move { .. } => {
                if !current_path.is_empty() {
                    paths.push(current_path.clone());
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
        paths.push(current_path);
    }

    paths
}
