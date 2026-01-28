use svgpath::Command;

fn main() {
    // SVG path d
    let s = "
    M 10,30
    A 20,20 0,0,1 50,30
    A 20,20 0,0,1 90,30
    Q 90,60 50,90
    Q 10,60 10,30
    Z";

    // Parse the string
    let p = svgpath::parse(s).unwrap();

    // Convert to SimplePath
    let sp = p.simplify();

    // Get the bounding box
    let bb = sp.bbox();
    println!("x: {}, y: {}", bb.min_x, bb.min_y);
    println!("width: {}, height: {}", bb.width(), bb.height());
    println!();

    // Scale and translate to fit inside 700 x 700 rectangle at X=50 and Y=50
    let rect = svgpath::Rect::new(50.0, 50.0, 700.0, 700.0);
    let sp = sp.fit(&rect, true, true);

    // Rotate 35 degree by its center point
    let center = sp.bbox().center();
    let m = svgpath::Matrix::new().rotate_by(35.0, center.x, center.y);
    let sp = sp.transform(&m);

    // print SVG path d
    println!("{sp}");
    println!();

    // Iterating over the commands
    for cmd in sp.commands() {
        // SimplePath consists only of absolute M, L, C and Z
        match *cmd {
            Command::Move { x, y } => println!("move {x:.2} {y:.2}"),
            Command::Line { x, y } => println!("line {x:.2} {y:.2}"),
            Command::Cubic {
                x1,
                y1,
                x2,
                y2,
                x,
                y,
            } => println!("cubic {x1:.2} {y1:.2} {x2:.2} {y2:.2} {x:.2} {y:.2}"),
            Command::Close => println!("close"),
            _ => {}
        }
    }
}
