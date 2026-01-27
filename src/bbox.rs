use crate::{Command, Point};

#[derive(Debug, Clone, PartialEq)]
pub struct BBox {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl Default for BBox {
    fn default() -> Self {
        Self::new()
    }
}

impl BBox {
    pub fn new() -> Self {
        Self {
            min_x: f64::INFINITY,
            min_y: f64::INFINITY,
            max_x: f64::NEG_INFINITY,
            max_y: f64::NEG_INFINITY,
        }
    }

    pub fn init(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    pub fn width(&self) -> f64 {
        self.max_x - self.min_x
    }

    pub fn height(&self) -> f64 {
        self.max_y - self.min_y
    }

    pub fn center(&self) -> Point {
        Point {
            x: self.min_x + (self.max_x - self.min_x) / 2.0,
            y: self.min_y + (self.max_y - self.min_y) / 2.0,
        }
    }

    fn add_point(&mut self, x: f64, y: f64) {
        if x < self.min_x {
            self.min_x = x;
        }
        if x > self.max_x {
            self.max_x = x;
        }
        if y < self.min_y {
            self.min_y = y;
        }
        if y > self.max_y {
            self.max_y = y;
        }
    }

    /// Expands the bounding box to enclose a cubic Bezier segment.
    fn add_cubic(&mut self, start: Point, cp1: Point, cp2: Point, end: Point) {
        // Always include the start and end points
        self.add_point(start.x, start.y);
        self.add_point(end.x, end.y);

        // Solve for extrema in X and Y dimensions independently
        self.add_bezier_extrema(start.x, cp1.x, cp2.x, end.x, true);
        self.add_bezier_extrema(start.y, cp1.y, cp2.y, end.y, false);
    }

    fn add_bezier_extrema(&mut self, p0: f64, p1: f64, p2: f64, p3: f64, is_x: bool) {
        // Derivative of cubic Bezier: at^2 + bt + c = 0
        let a = 3.0 * (-p0 + 3.0 * p1 - 3.0 * p2 + p3);
        let b = 6.0 * (p0 - 2.0 * p1 + p2);
        let c = 3.0 * (p1 - p0);

        let find_roots = |a: f64, b: f64, c: f64| {
            let mut roots = Vec::new();
            if a.abs() < 1e-9 {
                // Quadratic reduces to linear
                if b.abs() > 1e-9 {
                    roots.push(-c / b);
                }
            } else {
                let discriminant = b * b - 4.0 * a * c;
                if discriminant >= 0.0 {
                    let sqrt_d = discriminant.sqrt();
                    roots.push((-b + sqrt_d) / (2.0 * a));
                    roots.push((-b - sqrt_d) / (2.0 * a));
                }
            }
            roots
        };

        for t in find_roots(a, b, c) {
            if t > 0.0 && t < 1.0 {
                let mt = 1.0 - t;
                let val = mt * mt * mt * p0
                    + 3.0 * mt * mt * t * p1
                    + 3.0 * mt * t * t * p2
                    + t * t * t * p3;
                if is_x {
                    self.min_x = self.min_x.min(val);
                    self.max_x = self.max_x.max(val);
                } else {
                    self.min_y = self.min_y.min(val);
                    self.max_y = self.max_y.max(val);
                }
            }
        }
    }
}

pub(crate) fn bbox(commands: &[Command]) -> Option<BBox> {
    if commands.is_empty() {
        return None;
    }

    let mut bounds = BBox::new();
    let mut cursor = Point { x: 0.0, y: 0.0 };

    for cmd in commands {
        match *cmd {
            Command::Move { x, y } | Command::Line { x, y } => {
                bounds.add_point(x, y);
                cursor = Point { x, y };
            }
            Command::Cubic {
                x1,
                y1,
                x2,
                y2,
                x,
                y,
            } => {
                let cp1 = Point { x: x1, y: y1 };
                let cp2 = Point { x: x2, y: y2 };
                let end = Point { x, y };

                bounds.add_cubic(cursor, cp1, cp2, end);
                cursor = end;
            }
            Command::Close => {}
            _ => {}
        }
    }

    if bounds.min_x == f64::INFINITY {
        None
    } else {
        Some(bounds)
    }
}

#[cfg(test)]
mod t {
    use super::*;
    use crate::Command::*;

    #[test]
    fn bounding_box() {
        let p = [
            Move { x: 15.0, y: 10.0 },
            Line { x: 37.0, y: 10.0 },
            Line { x: 29.0, y: 134.0 },
        ];
        let bb = bbox(&p);
        assert!(bb.is_some());
        let bb = bb.unwrap();
        assert_eq!(bb, BBox::init(15.0, 10.0, 37.0, 134.0));
    }
}
