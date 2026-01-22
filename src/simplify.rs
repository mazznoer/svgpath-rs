use std::f64::consts::PI;

use crate::Command;
use crate::parser::Point;

pub(crate) fn simplify(commands: &[Command]) -> Vec<Command> {
    let mut simplified = Vec::with_capacity(commands.len());
    let mut cursor = Point { x: 0.0, y: 0.0 };
    let mut last_control_point: Option<Point> = None;

    for cmd in commands {
        match cmd {
            Command::Move { x, y } => {
                cursor = Point { x: *x, y: *y };
                last_control_point = None;
                simplified.push(Command::Move { x: *x, y: *y });
            }
            Command::Line { x, y } => {
                cursor = Point { x: *x, y: *y };
                last_control_point = None;
                simplified.push(Command::Line { x: *x, y: *y });
            }
            // Convert Horizontal to Line
            Command::Horizontal { x } => {
                cursor.x = *x;
                last_control_point = None;
                simplified.push(Command::Line {
                    x: cursor.x,
                    y: cursor.y,
                });
            }
            // Convert Vertical to Line
            Command::Vertical { y } => {
                cursor.y = *y;
                last_control_point = None;
                simplified.push(Command::Line {
                    x: cursor.x,
                    y: cursor.y,
                });
            }
            Command::Cubic {
                x1,
                y1,
                x2,
                y2,
                x,
                y,
            } => {
                cursor = Point { x: *x, y: *y };
                last_control_point = Some(Point { x: *x2, y: *y2 });
                simplified.push(Command::Cubic {
                    x1: *x1,
                    y1: *y1,
                    x2: *x2,
                    y2: *y2,
                    x: *x,
                    y: *y,
                });
            }
            // Convert Smooth Cubic to Cubic
            Command::SmoothCubic { x2, y2, x, y } => {
                let p1 = reflect(last_control_point, cursor);
                last_control_point = Some(Point { x: *x2, y: *y2 });
                cursor = Point { x: *x, y: *y };
                simplified.push(Command::Cubic {
                    x1: p1.x,
                    y1: p1.y,
                    x2: *x2,
                    y2: *y2,
                    x: *x,
                    y: *y,
                });
            }
            // Convert Quadratic to Cubic
            // Math: CP1 = Q0 + 2/3(Q1-Q0), CP2 = Q2 + 2/3(Q1-Q2)
            Command::Quadratic { x1, y1, x, y } => {
                let q1 = Point { x: *x1, y: *y1 };
                let q2 = Point { x: *x, y: *y };
                let cp1 = Point {
                    x: cursor.x + 2.0 / 3.0 * (q1.x - cursor.x),
                    y: cursor.y + 2.0 / 3.0 * (q1.y - cursor.y),
                };
                let cp2 = Point {
                    x: q2.x + 2.0 / 3.0 * (q1.x - q2.x),
                    y: q2.y + 2.0 / 3.0 * (q1.y - q2.y),
                };
                cursor = q2;
                last_control_point = Some(q1);
                simplified.push(Command::Cubic {
                    x1: cp1.x,
                    y1: cp1.y,
                    x2: cp2.x,
                    y2: cp2.y,
                    x: q2.x,
                    y: q2.y,
                });
            }
            // Convert Smooth Quadratic to Cubic
            Command::SmoothQuadratic { x, y } => {
                let q1 = reflect(last_control_point, cursor);
                let q2 = Point { x: *x, y: *y };
                let cp1 = Point {
                    x: cursor.x + 2.0 / 3.0 * (q1.x - cursor.x),
                    y: cursor.y + 2.0 / 3.0 * (q1.y - cursor.y),
                };
                let cp2 = Point {
                    x: q2.x + 2.0 / 3.0 * (q1.x - q2.x),
                    y: q2.y + 2.0 / 3.0 * (q1.y - q2.y),
                };
                cursor = q2;
                last_control_point = Some(q1);
                simplified.push(Command::Cubic {
                    x1: cp1.x,
                    y1: cp1.y,
                    x2: cp2.x,
                    y2: cp2.y,
                    x: q2.x,
                    y: q2.y,
                });
            }
            Command::Arc {
                rx,
                ry,
                x_axis_rotation,
                large_arc_flag,
                sweep_flag,
                x,
                y,
            } => {
                let target = Point { x: *x, y: *y };

                // Convert Arc to a series of Cubic Beziers
                let beziers = arc_to_cubics(
                    cursor,
                    *rx,
                    *ry,
                    *x_axis_rotation,
                    *large_arc_flag,
                    *sweep_flag,
                    target,
                );

                for b in beziers {
                    if let Command::Cubic { x2, y2, x, y, .. } = b {
                        last_control_point = Some(Point { x: x2, y: y2 });
                        cursor = Point { x, y };
                    }
                    simplified.push(b);
                }
            }
            Command::Close => {
                last_control_point = None;
                simplified.push(Command::Close);
            }
        }
    }
    simplified
}

fn reflect(last_cp: Option<Point>, cursor: Point) -> Point {
    match last_cp {
        Some(p) => Point {
            x: 2.0 * cursor.x - p.x,
            y: 2.0 * cursor.y - p.y,
        },
        None => cursor,
    }
}

fn arc_to_cubics(
    start: Point,
    mut rx: f64,
    mut ry: f64,
    x_axis_rot: f64,
    large_arc: bool,
    sweep: bool,
    end: Point,
) -> Vec<Command> {
    // Correct radii (SVG Spec Requirement)
    rx = rx.abs();
    ry = ry.abs();
    if rx == 0.0 || ry == 0.0 {
        return vec![Command::Line { x: end.x, y: end.y }];
    }

    // Coordinate transformation (Rotation to local space)
    let phi = x_axis_rot.to_radians();
    let cos_phi = phi.cos();
    let sin_phi = phi.sin();

    let dx = (start.x - end.x) / 2.0;
    let dy = (start.y - end.y) / 2.0;
    let x1p = cos_phi * dx + sin_phi * dy;
    let y1p = -sin_phi * dx + cos_phi * dy;

    // Ensure radii are large enough to reach the end point
    let lambda = (x1p * x1p) / (rx * rx) + (y1p * y1p) / (ry * ry);
    if lambda > 1.0 {
        let sqrt_lambda = lambda.sqrt();
        rx *= sqrt_lambda;
        ry *= sqrt_lambda;
    }

    // Find the Center Point (cx', cy') in local space
    let rx2 = rx * rx;
    let ry2 = ry * ry;
    let x1p2 = x1p * x1p;
    let y1p2 = y1p * y1p;

    let sign = if large_arc == sweep { -1.0 } else { 1.0 };
    let numerator = (rx2 * ry2 - rx2 * y1p2 - ry2 * x1p2).max(0.0);
    let denominator = rx2 * y1p2 + ry2 * x1p2;
    let coef = sign * (numerator / denominator).sqrt();

    let cxp = coef * (rx * y1p / ry);
    let cyp = coef * -(ry * x1p / rx);

    // Transform center back to global space
    let cx = cos_phi * cxp - sin_phi * cyp + (start.x + end.x) / 2.0;
    let cy = sin_phi * cxp + cos_phi * cyp + (start.y + end.y) / 2.0;

    // Calculate start angle and angle delta
    let start_vec = Point {
        x: (x1p - cxp) / rx,
        y: (y1p - cyp) / ry,
    };
    let end_vec = Point {
        x: (-x1p - cxp) / rx,
        y: (-y1p - cyp) / ry,
    };

    let theta1 = angle_between(Point { x: 1.0, y: 0.0 }, start_vec);
    let mut d_theta = angle_between(start_vec, end_vec);

    if !sweep && d_theta > 0.0 {
        d_theta -= 2.0 * PI;
    }
    if sweep && d_theta < 0.0 {
        d_theta += 2.0 * PI;
    }

    // Split into segments (max 90 degrees each)
    let segments_count = (d_theta.abs() / (PI / 2.0)).ceil() as u32;
    let delta = d_theta / segments_count as f64;
    let mut result = Vec::new();
    let mut current_theta = theta1;

    for _ in 0..segments_count {
        result.push(approximate_unit_bezier(
            cx,
            cy,
            rx,
            ry,
            phi,
            current_theta,
            delta,
        ));
        current_theta += delta;
    }

    result
}

/// Approximates a segment of an ellipse with a Cubic Bezier
fn approximate_unit_bezier(
    cx: f64,
    cy: f64,
    rx: f64,
    ry: f64,
    phi: f64,
    theta: f64,
    delta: f64,
) -> Command {
    let cos_phi = phi.cos();
    let sin_phi = phi.sin();

    let alpha = delta.sin() * ((4.0 / 3.0) * (1.0 - (delta / 2.0).cos()) / delta.sin());

    let cos_t = theta.cos();
    let sin_t = theta.sin();
    let cos_td = (theta + delta).cos();
    let sin_td = (theta + delta).sin();

    // Local coordinates of start, end, and control points
    let p1 = Point { x: cos_t, y: sin_t };
    let p2 = Point {
        x: cos_td,
        y: sin_td,
    };
    let cp1 = Point {
        x: p1.x - alpha * p1.y,
        y: p1.y + alpha * p1.x,
    };
    let cp2 = Point {
        x: p2.x + alpha * p2.y,
        y: p2.y - alpha * p2.x,
    };

    // Transform local points to global space (Scale + Rotate + Translate)
    let tr = |p: Point| -> (f64, f64) {
        let tx = p.x * rx;
        let ty = p.y * ry;
        (
            cos_phi * tx - sin_phi * ty + cx,
            sin_phi * tx + cos_phi * ty + cy,
        )
    };

    let (x1, y1) = tr(cp1);
    let (x2, y2) = tr(cp2);
    let (x, y) = tr(p2);

    Command::Cubic {
        x1,
        y1,
        x2,
        y2,
        x,
        y,
    }
}

fn angle_between(v1: Point, v2: Point) -> f64 {
    let dot = v1.x * v2.x + v1.y * v2.y;
    let det = v1.x * v2.y - v1.y * v2.x;
    det.atan2(dot)
}
