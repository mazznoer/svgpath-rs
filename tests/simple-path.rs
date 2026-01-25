use svgpath::BBox;

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
fn tests() {
    struct Data<'a> {
        // input path
        input: &'a str,
        // Path
        path: &'a str,
        // SimplePath
        simple: &'a str,
        // bounding box
        bbox: svgpath::BBox,
        // fit to box
        fit: &'a str,
        // transforms
        transform: &'a str,
    }

    let test_data = [Data {
        input: "
            M 10,30
            A 20,20 0,0,1 50,30
            A 20,20 0,0,1 90,30
            Q 90,60 50,90
            Q 10,60 10,30
            Z",
        path: "\
            M 10 30 \
            A 20 20 0 0 1 50 30 \
            A 20 20 0 0 1 90 30 \
            Q 90 60 50 90 \
            Q 10 60 10 30 \
            Z",
        simple: "\
            M 10 30 \
            C 10 18.95 18.95 10 30 10 \
            C 41.05 10 50 18.95 50 30 \
            C 50 18.95 58.95 10 70 10 \
            C 81.05 10 90 18.95 90 30 \
            C 90 50 76.67 70 50 90 \
            C 23.33 70 10 50 10 30 \
            Z",
        bbox: BBox::init(10.0, 10.0, 90.0, 90.0),
        fit: "\
            M 50 225 \
            C 50 128.35 128.35 50 225 50 \
            C 321.65 50 400 128.35 400 225 \
            C 400 128.35 478.35 50 575 50 \
            C 671.65 50 750 128.35 750 225 \
            C 750 400 633.33 575 400 750 \
            C 166.67 575 50 400 50 225 \
            Z",
        transform: "\
            M 349.57 338.99 \
            C 231.69 238.04 144.55 203.5 154.93 261.84 \
            C 165.32 320.19 269.3 449.31 387.18 550.26 \
            C 269.3 449.31 182.16 414.78 192.55 473.12 \
            C 202.93 531.46 306.92 660.59 424.8 761.53 \
            C 638.24 944.31 839.15 1056.67 1027.52 1098.6 \
            C 789 774.97 563.02 521.77 349.57 338.99 \
            Z",
    }];

    let rect = svgpath::Rect::new(50.0, 50.0, 700.0, 700.0);

    let m = svgpath::Matrix::new()
        .translate(20.0, 12.0)
        .scale(1.75, 2.1)
        .rotate(25.0)
        .skew_x(15.0)
        .skew_y(3.0)
        .shear(7.0, 2.0);

    for d in test_data {
        let p = svgpath::parse(d.input);
        assert!(p.is_ok());
        let p = p.unwrap();
        assert_eq!(p.to_string(), d.path);

        let mut sp = p.simplify();
        assert_eq!(sp.to_string(), d.simple);
        assert_eq!(sp.bbox(), td.bbox);

        let fp = sp.fit(&rect, true, true);
        assert_eq!(fp.to_string(), d.fit);

        let tp = sp.transform(&m);
        assert_eq!(tp.to_string(), d.transform);
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
