enum StringTest {
    Empty,
    AtLeast(&'static str),
    Exactly(&'static str),
}

use StringTest::*;

impl StringTest {
    fn test(&self, actual: &str, prefix: &str) {
        match self {
            Empty => assert!(
                actual.is_empty(),
                format!("{}: expected empty, got\n\n{}", prefix, actual)
            ),
            AtLeast(exp) => assert!(
                actual.contains(exp),
                format!(
                    "{}: expected at least\n\n{}\n\ngot\n\n{}",
                    prefix, exp, actual
                )
            ),
            Exactly(exp) => assert!(
                actual.trim() == exp.trim(),
                format!("{}: expected\n\n{}\n\ngot \n\n{}", prefix, exp, actual)
            ),
        }
    }
}

struct ProcessTest {
    args: &'static [&'static str],
    out: StringTest,
    err: StringTest,
    is_success: bool,
}

// fn test_cli(args: &[&str], is_success: bool, out: &str, err: &str) {
fn test_cli(descr: ProcessTest) {
    let mut cmd = &mut std::process::Command::new("diffr");
    for arg in descr.args {
        cmd = cmd.arg(&*arg);
    }
    let output = cmd.output().unwrap();
    fn string_of_status(code: bool) -> &'static str {
        if code {
            "success"
        } else {
            "failure"
        }
    };
    assert!(
        descr.is_success == output.status.success(),
        format!(
            "unexpected status: expected\n\n{}\n\ngot \n\n{}",
            string_of_status(descr.is_success),
            string_of_status(output.status.success()),
        )
    );
    descr
        .out
        .test(&String::from_utf8_lossy(&output.stdout), "stdout");
    descr
        .err
        .test(&String::from_utf8_lossy(&output.stderr), "stderr");
}

#[test]
fn debug_flag() {
    test_cli(ProcessTest {
        args: &["--debug"],
        out: Empty,
        err: AtLeast("hunk processing time (ms):"),
        is_success: true,
    })
}

#[test]
fn color_invalid_face_name() {
    test_cli(ProcessTest {
        args: &["--colors", "notafacename"],
        out: Empty,
        err: Exactly("unexpected face name: got 'notafacename', expected added|refine-added|removed|refine-removed"),
        is_success: false,
    })
}

#[test]
fn color_only_face_name() {
    test_cli(ProcessTest {
        args: &["--colors", "added"],
        out: Empty,
        err: Exactly(""),
        is_success: true,
    })
}

#[test]
fn color_invalid_attribute_name() {
    test_cli(ProcessTest {
        args: &["--colors", "added:bar"],
        out: Empty,
        err: Exactly("unexpected attribute name: got 'bar', expected foreground|background|bold|nobold|intense|nointense|underline|nounderline|none"),
        is_success: false,
    })
}

#[test]
fn color_invalid_color_value_name() {
    test_cli(ProcessTest {
        args: &["--colors", "added:foreground:baz"],
        out: Empty,
        err: Exactly("unexpected color value: unrecognized color name 'baz'. Choose from: black, blue, green, red, cyan, magenta, yellow, white"),
        is_success: false,
    })
}

#[test]
fn color_invalid_color_value_ansi() {
    test_cli(ProcessTest {
        args: &["--colors", "added:foreground:777"],
        out: Empty,
        err: AtLeast("unexpected color value: unrecognized ansi256 color number"),
        is_success: false,
    })
}

#[test]
fn color_invalid_color_value_rgb() {
    test_cli(ProcessTest {
        args: &["--colors", "added:foreground:0,0,777"],
        out: Empty,
        err: AtLeast("unexpected color value: unrecognized RGB color triple"),
        is_success: false,
    })
}

#[test]
fn color_invalid_color_not_done() {
    test_cli(ProcessTest {
        args: &["--colors", "added:foreground"],
        out: Empty,
        err: Exactly("error parsing color: missing color value for face 'added'"),
        is_success: false,
    })
}

#[test]
fn color_ok() {
    test_cli(ProcessTest {
        args: &["--colors", "added:foreground:0"],
        out: Empty,
        err: Exactly(""),
        is_success: true,
    })
}

#[test]
fn color_ok_multiple() {
    test_cli(ProcessTest {
        args: &[
            "--colors",
            "added:foreground:0",
            "--colors",
            "removed:background:red",
        ],
        out: Empty,
        err: Exactly(""),
        is_success: true,
    })
}