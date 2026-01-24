#[test]
fn basic() {
    let test_data = [
        ["M3,5v-7h10", "M 3 5 L 3 -2 L 13 -2"],
        ["M10,5h15v-7z", "M 10 5 L 25 5 L 25 -2 Z"],
    ];
    for [input, output] in test_data {
        let p = svgpath::parse(input);
        assert!(p.is_ok());
        let sp = p.unwrap().simplify();
        assert_eq!(sp.to_string(), output);
    }
}

#[test]
fn transform() {
    let s = "M 3 2 L 7 2 L 7 0 Z";
    let p = svgpath::parse(s);
    assert!(p.is_ok());
    let sp = p.unwrap().simplify();

    let m = svgpath::Matrix::new().translate(10.0, 5.0);
    let res = sp.transform(&m);
    assert_eq!(res.to_string(), "M 13 7 L 17 7 L 17 5 Z");
}
