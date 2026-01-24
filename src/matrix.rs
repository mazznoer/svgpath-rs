use crate::{Command, Point};

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
    fn transform_point(&self, p: Point) -> Point {
        Point {
            x: self.a * p.x + self.c * p.y + self.e,
            y: self.b * p.x + self.d * p.y + self.f,
        }
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

pub(crate) fn transform_path(commands: &[Command], matrix: &Matrix) -> Vec<Command> {
    commands
        .iter()
        .filter_map(|cmd| match cmd {
            Command::Move { x, y } => {
                let p = matrix.transform_point(Point { x: *x, y: *y });
                Some(Command::Move { x: p.x, y: p.y })
            }
            Command::Line { x, y } => {
                let p = matrix.transform_point(Point { x: *x, y: *y });
                Some(Command::Line { x: p.x, y: p.y })
            }
            Command::Cubic {
                x1,
                y1,
                x2,
                y2,
                x,
                y,
            } => {
                let p1 = matrix.transform_point(Point { x: *x1, y: *y1 });
                let p2 = matrix.transform_point(Point { x: *x2, y: *y2 });
                let p = matrix.transform_point(Point { x: *x, y: *y });
                Some(Command::Cubic {
                    x1: p1.x,
                    y1: p1.y,
                    x2: p2.x,
                    y2: p2.y,
                    x: p.x,
                    y: p.y,
                })
            }
            Command::Close => Some(Command::Close),
            _ => None,
        })
        .collect()
}
