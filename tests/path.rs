use svgpath::Command;

#[test]
fn basic() {
    let test_data = [
        ["M 1 2 3 4 5 6", "M 1 2 L 3 4 L 5 6"],
        ["M 1 2 L 3 4 5 6 7 8", "M 1 2 L 3 4 L 5 6 L 7 8"],
        ["M 7,9 L 100,75 h -50 z", "M 7 9 L 100 75 H 50 Z"],
        ["M3,5v-7h10", "M 3 5 V -2 H 13"],
        [" M10-20  ", "M 10 -20"],
        ["M 5,7 L 10 10 20 20 55,75", "M 5 7 L 10 10 L 20 20 L 55 75"],
        ["M10,5h15v-7z", "M 10 5 H 25 V -2 Z"],
        ["M 0.012,0 L 95.1205 7.09420001", "M 0.01 0 L 95.12 7.09"],
    ];
    for [input, output] in test_data {
        let p = svgpath::parse(input);
        assert!(p.is_ok());
        let p = p.unwrap();
        assert_eq!(p.to_string(), output);
    }

    let s = "M 7,9 L 100,75 h -50 z";
    let p = svgpath::parse(s);
    assert!(p.is_ok());
    let p = p.unwrap();

    let mut it = p.iter();
    assert_eq!(it.next(), Some(&Command::Move { x: 7.0, y: 9.0 }));
    assert_eq!(it.next(), Some(&Command::Line { x: 100.0, y: 75.0 }));
    assert_eq!(it.next(), Some(&Command::Horizontal { x: 50.0 }));
    assert_eq!(it.next(), Some(&Command::Close));
    assert_eq!(it.next(), None);
}

#[test]
fn split() {
    let s = "M 25 67 H 90 V 150 M 5 7 L 90 55";
    let p = svgpath::parse(s);
    assert!(p.is_ok());
    let p = p.unwrap();

    let paths = p.split();
    assert_eq!(paths.len(), 2);
    assert_eq!(paths[0].to_string(), "M 25 67 H 90 V 150");
    assert_eq!(paths[1].to_string(), "M 5 7 L 90 55");
}

#[test]
fn invalid() {
    let test_data = [
        "",
        "  \n \t ",
        "5",
        "M",
        "M 7",
        "M 0,0 L",
        "M 3 5 L 5",
        "M 5 5 H 10 X 7 3 Z",
        "M 3 4 5 H 10 Z",
        //"L 15 37 v 30 h 100 z",
    ];
    for s in test_data {
        let p = svgpath::parse(s);
        assert!(p.is_err());
    }
}
