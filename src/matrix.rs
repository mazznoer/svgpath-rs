use std::fmt;

use crate::Command;
use crate::parser::format_n;

#[derive(Debug, Clone, PartialEq)]
pub struct Matrix {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub e: f64,
    pub f: f64,
}

impl Default for Matrix {
    fn default() -> Self {
        Self::new()
    }
}

impl Matrix {
    /// Returns the identity matrix
    pub fn new() -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        }
    }

    /// Transforms a point: x' = ax + cy + e, y' = bx + dy + f
    fn transform_point(&self, x: f64, y: f64) -> [f64; 2] {
        [
            self.a * x + self.c * y + self.e,
            self.b * x + self.d * y + self.f,
        ]
    }

    /// Multiply two matrices (Combine transformations)
    #[must_use]
    pub fn multiply(&self, other: &Matrix) -> Self {
        Self {
            a: self.a * other.a + self.c * other.b,
            b: self.b * other.a + self.d * other.b,
            c: self.a * other.c + self.c * other.d,
            d: self.b * other.c + self.d * other.d,
            e: self.a * other.e + self.c * other.f + self.e,
            f: self.b * other.e + self.d * other.f + self.f,
        }
    }

    #[must_use]
    pub fn translate(&self, tx: f64, ty: f64) -> Self {
        let m = Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: tx,
            f: ty,
        };
        self.multiply(&m)
    }

    #[must_use]
    pub fn scale(&self, sx: f64, sy: f64) -> Self {
        let m = Self {
            a: sx,
            b: 0.0,
            c: 0.0,
            d: sy,
            e: 0.0,
            f: 0.0,
        };
        self.multiply(&m)
    }

    #[must_use]
    pub fn rotate(&self, angle_deg: f64) -> Self {
        let rad = angle_deg.to_radians();
        let cos = rad.cos();
        let sin = rad.sin();
        let m = Self {
            a: cos,
            b: sin,
            c: -sin,
            d: cos,
            e: 0.0,
            f: 0.0,
        };
        self.multiply(&m)
    }

    #[must_use]
    pub fn rotate_by(&self, angle_deg: f64, x: f64, y: f64) -> Self {
        self.translate(x, y).rotate(angle_deg).translate(-x, -y)
    }

    #[must_use]
    pub fn skew_x(&self, angle_deg: f64) -> Self {
        let m = Self {
            a: 1.0,
            b: 0.0,
            c: angle_deg.to_radians().tan(),
            d: 1.0,
            e: 0.0,
            f: 0.0,
        };
        self.multiply(&m)
    }

    #[must_use]
    pub fn skew_y(&self, angle_deg: f64) -> Self {
        let m = Self {
            a: 1.0,
            b: angle_deg.to_radians().tan(),
            c: 0.0,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        };
        self.multiply(&m)
    }

    #[must_use]
    pub fn shear(&self, x: f64, y: f64) -> Self {
        let m = Self {
            a: 1.0,
            b: y,
            c: x,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        };
        self.multiply(&m)
    }
}

impl fmt::Display for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "matrix({} {} {} {} {} {})",
            format_n(self.a),
            format_n(self.b),
            format_n(self.c),
            format_n(self.d),
            format_n(self.e),
            format_n(self.f)
        )
    }
}

pub(crate) fn transform_path(commands: &[Command], matrix: &Matrix) -> Vec<Command> {
    commands
        .iter()
        .filter_map(|cmd| match *cmd {
            Command::Move { x, y } => {
                let [x, y] = matrix.transform_point(x, y);
                Some(Command::Move { x, y })
            }
            Command::Line { x, y } => {
                let [x, y] = matrix.transform_point(x, y);
                Some(Command::Line { x, y })
            }
            Command::Cubic {
                x1,
                y1,
                x2,
                y2,
                x,
                y,
            } => {
                let [x1, y1] = matrix.transform_point(x1, y1);
                let [x2, y2] = matrix.transform_point(x2, y2);
                let [x, y] = matrix.transform_point(x, y);
                Some(Command::Cubic {
                    x1,
                    y1,
                    x2,
                    y2,
                    x,
                    y,
                })
            }
            Command::Close => Some(Command::Close),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod t {
    use super::*;

    #[test]
    fn matrix() {
        let a = Matrix::new()
            .translate(15.0, 7.0)
            .rotate(34.0)
            .translate(-15.0, -7.0);
        let b = Matrix::new().rotate_by(34.0, 15.0, 7.0);
        assert_eq!(a, b);
        assert_eq!(a.to_string(), b.to_string());

        let m = Matrix::new()
            .rotate_by(-10.0, 50.0, 100.0)
            .translate(-36.0, 45.5)
            .skew_x(40.0)
            .scale(1.0, 0.5);
        assert_eq!(m.to_string(), "matrix(0.98 -0.17 0.5 0.42 -44.16 61.26)");
    }
}
