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

    /// Parses an SVG transform string
    pub fn parse(input: &str) -> Result<Self, String> {
        let mut result = Matrix::new();

        // Replace commas/parens with spaces to easily tokenize,
        // but keep the function names identifiable.
        let normalized = input
            .replace(',', " ")
            .replace('(', " ( ")
            .replace(')', " ) ");

        let mut tokens = normalized.split_whitespace().peekable();

        while let Some(func_name) = tokens.next() {
            // Expect an opening parenthesis next
            match tokens.next() {
                Some("(") => {}
                _ => return Err(format!("Expected '(' after {}", func_name)),
            }

            // Collect all numbers until the closing parenthesis
            let mut nums = Vec::new();
            while let Some(&next_token) = tokens.peek() {
                if next_token == ")" {
                    // Consume ')'
                    tokens.next();
                    break;
                }
                let n: f64 = tokens
                    .next()
                    .unwrap()
                    .parse()
                    .map_err(|_| format!("Invalid number in {}", func_name))?;
                nums.push(n);
            }

            if nums.is_empty() {
                return Err(format!("No arguments provided for {}", func_name));
            }

            match func_name.to_lowercase().as_str() {
                "matrix" if nums.len() == 6 => {
                    let m = Matrix {
                        a: nums[0],
                        b: nums[1],
                        c: nums[2],
                        d: nums[3],
                        e: nums[4],
                        f: nums[5],
                    };
                    result = result.multiply(&m);
                }
                "translate" => {
                    let tx = nums[0];
                    let ty = nums.get(1).cloned().unwrap_or(0.0);
                    result = result.translate(tx, ty);
                }
                "scale" => {
                    let sx = nums[0];
                    let sy = nums.get(1).cloned().unwrap_or(sx);
                    result = result.scale(sx, sy);
                }
                "rotate" => {
                    let angle = nums[0];
                    if nums.len() == 3 {
                        result = result.rotate_by(angle, nums[1], nums[2]);
                    } else {
                        result = result.rotate(angle);
                    }
                }
                "skewx" => {
                    result = result.skew_x(nums[0]);
                }
                "skewy" => {
                    result = result.skew_y(nums[0]);
                }
                _ => return Err(format!("Unknown or invalid transform: {}", func_name)),
            };
        }

        Ok(result)
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

    #[test]
    fn parse_str() {
        let m1 = Matrix::new()
            .translate(5.0, 13.0)
            .scale(1.75, 1.75)
            .rotate_by(35.0, 100.0, 75.0)
            .translate(-8.0, -10.0);

        let s = "
        translate(5, 13)
        scale(1.75)
        rotate(35, 100, 75)
        translate(-8.0, -10)
        ";

        let m2 = Matrix::parse(s);
        if m2.is_err() {
            println!("{:?}", m2.clone().unwrap_err());
        }
        assert!(m2.is_ok());
        let m2 = m2.unwrap();
        assert_eq!(m1, m2);
        assert_eq!(m1.to_string(), m2.to_string());
    }
}
