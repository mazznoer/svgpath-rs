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
    rx = rx.abs();
    ry = ry.abs();
    if rx < 1e-6 || ry < 1e-6 {
        return vec![Command::Line { x: end.x, y: end.y }];
    }

    // Center Parameterization (Simplified for precision)
    let phi = x_axis_rot.to_radians();
    let cos_phi = phi.cos();
    let sin_phi = phi.sin();

    let x1p = cos_phi * (start.x - end.x) / 2.0 + sin_phi * (start.y - end.y) / 2.0;
    let y1p = -sin_phi * (start.x - end.x) / 2.0 + cos_phi * (start.y - end.y) / 2.0;

    let rx2 = rx * rx;
    let ry2 = ry * ry;
    let x1p2 = x1p * x1p;
    let y1p2 = y1p * y1p;

    let check = x1p2 / rx2 + y1p2 / ry2;
    if check > 1.0 {
        rx *= check.sqrt();
        ry *= check.sqrt();
    }

    let sign = if large_arc == sweep { -1.0 } else { 1.0 };
    let n = (rx * rx * ry * ry - rx * rx * y1p2 - ry * ry * x1p2).max(0.0);
    let d = rx * rx * y1p2 + ry * ry * x1p2;
    let coef = sign * (n / d).sqrt();

    let cxp = coef * rx * y1p / ry;
    let cyp = coef * -ry * x1p / rx;

    let cx = cos_phi * cxp - sin_phi * cyp + (start.x + end.x) / 2.0;
    let cy = sin_phi * cxp + cos_phi * cyp + (start.y + end.y) / 2.0;

    // Angle Calculations
    let theta1 = angle_between(1.0, 0.0, (x1p - cxp) / rx, (y1p - cyp) / ry);
    let mut d_theta = angle_between(
        (x1p - cxp) / rx,
        (y1p - cyp) / ry,
        (-x1p - cxp) / rx,
        (-y1p - cyp) / ry,
    );

    if !sweep && d_theta > 0.0 {
        d_theta -= 2.0 * PI;
    }
    if sweep && d_theta < 0.0 {
        d_theta += 2.0 * PI;
    }

    // Precise Splitting
    let segments = (d_theta.abs() / (PI / 2.0 + 0.001)).ceil() as u32;
    let delta = d_theta / segments as f64;
    let mut result = Vec::new();

    for i in 0..segments {
        let t_start = theta1 + i as f64 * delta;
        result.push(single_arc_segment(cx, cy, rx, ry, phi, t_start, delta));
    }

    result
}

fn single_arc_segment(
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

    // The precise "Kappa" for this specific angular delta
    let kappa = (delta / 4.0).tan() * 4.0 / 3.0;

    let t1 = theta;
    let t2 = theta + delta;

    let (cos1, sin1) = (t1.cos(), t1.sin());
    let (cos2, sin2) = (t2.cos(), t2.sin());

    // Local coordinates
    let p1 = Point { x: cos1, y: sin1 };
    let p2 = Point { x: cos2, y: sin2 };

    // Derivative vectors for control points
    let q1 = Point {
        x: -kappa * sin1,
        y: kappa * cos1,
    };
    let q2 = Point {
        x: -kappa * sin2,
        y: kappa * cos2,
    };

    let cp1 = Point {
        x: p1.x + q1.x,
        y: p1.y + q1.y,
    };
    let cp2 = Point {
        x: p2.x - q2.x,
        y: p2.y - q2.y,
    };

    let tr = |p: Point| -> (f64, f64) {
        let x = p.x * rx;
        let y = p.y * ry;
        (
            cos_phi * x - sin_phi * y + cx,
            sin_phi * x + cos_phi * y + cy,
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

fn angle_between(ux: f64, uy: f64, vx: f64, vy: f64) -> f64 {
    let dot = ux * vx + uy * vy;
    let det = ux * vy - uy * vx;
    det.atan2(dot)
}
