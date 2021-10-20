use const_panic::FmtArg;

const DISPLAY: FmtArg = FmtArg::DISPLAY;
const DISPLAY_TUP: (FmtArg,) = (FmtArg::DISPLAY,);
const DEBUG_TUP2: (((), FmtArg),) = (((), FmtArg::DEBUG),);

#[test]
fn concat_panic_args_test() {
    let string = "\nfoo\"\r";
    let string_debug = r#""\nfoo\"\r""#;

    assert_eq!(
        trunc_fmt!(999; ::const_panic::FmtArg::DEBUG; "\nfoo\"\r"),
        string
    );
    assert_eq!(trunc_fmt!(999; FmtArg::DEBUG; string), string_debug);

    assert_eq!(trunc_fmt!(999; DISPLAY; "\nfoo\"\r"), string);
    assert_eq!(trunc_fmt!(999; DISPLAY_TUP.0; string), string);
    assert_eq!(trunc_fmt!(999; DEBUG_TUP2.0.1; string), string_debug);

    assert_eq!(trunc_fmt!(999; FmtArg::DISPLAY; "\nfoo\"\r"), string);
    assert_eq!(
        trunc_fmt!(999; const_panic::FmtArg::DISPLAY; string),
        string
    );

    assert_eq!(trunc_fmt!(999; "\nfoo\"\r"), string);
    assert_eq!(trunc_fmt!(999; string), string_debug);

    assert_eq!(trunc_fmt!(999; debug: "\nfoo\"\r"), string_debug);
    assert_eq!(trunc_fmt!(999; debug: string), string_debug);

    assert_eq!(trunc_fmt!(999; display: "\nfoo\"\r"), string);
    assert_eq!(trunc_fmt!(999; display: string), string);
}

macro_rules! fmt_flatten {
    ($($args:tt)*) => (
        const_panic::ArrayString::<256>::from_panicvals(
            &const_panic::flatten_panicvals!($($args)*)
        ).unwrap()
    )
}

#[test]
fn flatten_panicvals_args_test() {
    let string = "\nfoo\"\r";
    let string_debug = r#""\nfoo\"\r""#;

    assert_eq!(
        fmt_flatten!(::const_panic::FmtArg::DEBUG; "\nfoo\"\r"),
        string
    );
    assert_eq!(
        fmt_flatten!(const_panic::FmtArg::DEBUG; string),
        string_debug
    );

    assert_eq!(fmt_flatten!(DISPLAY; "\nfoo\"\r"), string);
    assert_eq!(fmt_flatten!(DISPLAY_TUP.0; string), string);
    assert_eq!(fmt_flatten!(DEBUG_TUP2.0.1; string), string_debug);

    assert_eq!(fmt_flatten!(FmtArg::DISPLAY; "\nfoo\"\r"), string);
    assert_eq!(fmt_flatten!(FmtArg::DISPLAY; string), string);

    assert_eq!(
        fmt_flatten!(FmtArg::DEBUG; debug: "\nfoo\"\r"),
        string_debug
    );
    assert_eq!(fmt_flatten!(FmtArg::DEBUG; debug: string), string_debug);

    assert_eq!(fmt_flatten!(FmtArg::DEBUG; display: "\nfoo\"\r"), string);
    assert_eq!(fmt_flatten!(FmtArg::DEBUG; display: string), string);

    assert_eq!(fmt_flatten!(FmtArg::DEBUG; &str => "\nfoo\"\r"), string);
    assert_eq!(
        fmt_flatten!(FmtArg::DEBUG; &str => display: "\nfoo\"\r"),
        string
    );
    assert_eq!(
        fmt_flatten!(FmtArg::DEBUG; &str => debug: "\nfoo\"\r"),
        string_debug
    );
    assert_eq!(fmt_flatten!(FmtArg::DEBUG; &str => string), string_debug);
    assert_eq!(fmt_flatten!(FmtArg::DISPLAY; &str => string), string);
    assert_eq!(fmt_flatten!(FmtArg::DEBUG; &str => display: string), string);
    assert_eq!(
        fmt_flatten!(FmtArg::DEBUG; &str => debug: string),
        string_debug
    );
}

#[cfg(feature = "non_basic")]
#[test]
fn struct_formatting() {
    let foo = Foo {
        x: &[3, 5, 8, 13],
        y: 21,
        z: Bar(false, true),
        w: Baz {
            h: &["hi", "hel\nlo"],
        },
    };

    assert_eq!(
        fmt_flatten!(FmtArg::DEBUG; Foo => foo),
        *format!("{:?}", foo)
    );

    assert_eq!(
        fmt_flatten!(FmtArg::ALT_DEBUG; Foo => foo),
        *format!("{:#?}", foo)
    );

    assert_eq!(
        fmt_flatten!(FmtArg::DISPLAY; Foo => foo),
        "Foo { x: [3, 5, 8, 13], y: 21, z: Bar(false, true), w: Baz { h: [hi, hel\nlo] } }"
    );

    assert_eq!(
        fmt_flatten!(FmtArg::ALT_DISPLAY; Foo => foo),
        "\
Foo {
    x: [
        3,
        5,
        8,
        13,
    ],
    y: 21,
    z: Bar(
        false,
        true,
    ),
    w: Baz {
        h: [
            hi,
            hel
lo,
        ],
    },
}\
"
    );
}

#[cfg(feature = "non_basic")]
#[derive(Debug)]
struct Foo<'a> {
    x: &'a [u8],
    y: u8,
    z: Bar,
    w: Baz,
}

#[cfg(feature = "non_basic")]
#[derive(Debug)]
struct Bar(bool, bool);

#[cfg(feature = "non_basic")]
#[derive(Debug)]
struct Baz {
    h: &'static [&'static str],
}

#[cfg(feature = "non_basic")]
const _: () = {
    use const_panic::PanicFmt;

    impl<'a> Foo<'a> {
        const fn to_panicvals(&self, f: FmtArg) -> [const_panic::PanicVal<'a>; Foo::PV_COUNT] {
            use const_panic::fmt;
            const_panic::flatten_panicvals! {f;
                "Foo",
                open: fmt::OPEN_BRACE,
                    "x: ", &[u8] => self.x, fmt::COMMA_SEP,
                    "y: ", u8 => self.y, fmt::COMMA_SEP,
                    "z: ", Bar => self.z, fmt::COMMA_SEP,
                    "w: ", Baz => self.w, fmt::COMMA_TERM,
                close: fmt::CLOSE_BRACE,
            }
        }
    }

    impl Bar {
        const fn to_panicvals(&self, f: FmtArg) -> [const_panic::PanicVal<'static>; Bar::PV_COUNT] {
            use const_panic::fmt;
            const_panic::flatten_panicvals! {f;
                "Bar",
                open: fmt::OPEN_PAREN,
                    self.0, fmt::COMMA_SEP,
                    self.1, fmt::COMMA_TERM,
                close: fmt::CLOSE_PAREN,
            }
        }
    }

    impl Baz {
        const fn to_panicvals(&self, f: FmtArg) -> [const_panic::PanicVal<'static>; Baz::PV_COUNT] {
            use const_panic::fmt;
            const_panic::flatten_panicvals! {f;
                "Baz",
                open: fmt::OPEN_BRACE,
                    "h: ", self.h, fmt::COMMA_TERM,
                close: fmt::CLOSE_BRACE,
            }
        }
    }

    impl PanicFmt for Foo<'_> {
        type This = Self;
        type Kind = const_panic::fmt::IsCustomType;

        const PV_COUNT: usize = {
            let name = 1;
            let open_brace = 1;
            let close_brace = 1;
            let field_count = 4;
            name + open_brace
                + close_brace
                + 2 * field_count
                + <&[u8]>::PV_COUNT
                + <u8>::PV_COUNT
                + <Bar>::PV_COUNT
                + <Baz>::PV_COUNT
        };
    }

    impl PanicFmt for Bar {
        type This = Self;
        type Kind = const_panic::fmt::IsCustomType;

        const PV_COUNT: usize = 7;
    }

    impl PanicFmt for Baz {
        type This = Self;
        type Kind = const_panic::fmt::IsCustomType;

        const PV_COUNT: usize = 6;
    }
};
