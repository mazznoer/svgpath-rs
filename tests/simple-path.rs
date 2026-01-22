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
