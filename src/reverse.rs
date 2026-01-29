use crate::{Command, Point};

pub(crate) fn reverse_path(commands: &[Command]) -> Vec<Command> {
    if commands.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::new();
    let mut subpath = Vec::new();

    for cmd in commands {
        if matches!(cmd, Command::Move { .. }) && !subpath.is_empty() {
            result.extend(reverse_subpath(&subpath));
            subpath.clear();
        }
        subpath.push(cmd);
    }

    if !subpath.is_empty() {
        result.extend(reverse_subpath(&subpath));
    }

    result
}

fn reverse_subpath(cmds: &[&Command]) -> Vec<Command> {
    if cmds.is_empty() {
        return vec![];
    }

    // Trace points forward
    let mut points = Vec::new();
    let mut cursor = Point { x: 0.0, y: 0.0 };
    let mut start_pt = Point { x: 0.0, y: 0.0 };

    for cmd in cmds {
        match **cmd {
            Command::Move { x, y } => {
                cursor = Point { x, y };
                start_pt = cursor;
            }
            Command::Line { x, y } | Command::Cubic { x, y, .. } => {
                cursor = Point { x, y };
            }
            Command::Close => {
                cursor = start_pt;
            }
            _ => {}
        }
        points.push(cursor);
    }

    // Build reversed subpath
    let mut reversed = Vec::new();
    let last_pt = points.last().expect("Subpath points cannot be empty");

    // Start with a Move to the end of the original subpath
    reversed.push(Command::Move {
        x: last_pt.x,
        y: last_pt.y,
    });
    let mut current_pos = *last_pt;

    // Iterate backwards through segments
    for i in (1..cmds.len()).rev() {
        let cmd = cmds[i];
        let prev_pt = points[i - 1]; // The 'start' of this command in forward direction

        match *cmd {
            Command::Line { .. } | Command::Close => {
                // Eliminate the 'L x y' if we are already at (x, y)
                if (prev_pt.x - current_pos.x).abs() > 1e-9
                    || (prev_pt.y - current_pos.y).abs() > 1e-9
                {
                    reversed.push(Command::Line {
                        x: prev_pt.x,
                        y: prev_pt.y,
                    });
                    current_pos = prev_pt;
                }
            }
            Command::Cubic { x1, y1, x2, y2, .. } => {
                reversed.push(Command::Cubic {
                    x1: x2,
                    y1: y2, // Control points must be swapped
                    x2: x1,
                    y2: y1,
                    x: prev_pt.x,
                    y: prev_pt.y,
                });
                current_pos = prev_pt;
            }
            _ => {}
        }
    }

    // If the forward subpath ended with a ClosePath, the reversed one should too
    if let Some(Command::Close) = cmds.last() {
        reversed.push(Command::Close);
    }

    reversed
}
