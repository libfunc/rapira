#[macro_export]
macro_rules! ser {
    ($(($i:expr, $s:path)),* $(| $($t:expr),*)?) => {{
        let size = {
            0
            $(
                + {
                    use $s as ss;
                    ss::size($i)
                }
            )*
            $($(
                + rapira::Rapira::size($t)
            )*)?
        };

        let mut bytes = vec![0u8; size];
        let mut cursor = 0usize;

        $(
            {
                use $s as ss;
                ss::convert_to_bytes($i, &mut bytes, &mut cursor);
            };
        )*

        $($(
            rapira::Rapira::convert_to_bytes($t, &mut bytes, &mut cursor);
        )*)?

        bytes
    }};
}

#[macro_export]
macro_rules! deser {
    ($bytes:expr, $($s:path),* $(| $($t:ty),*)?) => {{
        let bytes = &mut $bytes;
        (
            $(
                {
                    use $s as ss;
                    ss::from_slice(bytes).unwrap()
                },
            )*
            $($(
                {
                    <$t as rapira::Rapira>::from_slice(bytes).unwrap()
                },
            )*)?
        )
    }};
}

#[test]
fn ser_test() {
    use crate as rapira;
    use crate::str_rapira;

    let b = "a123456789";
    let c = 3u8;
    let a = 10u32;

    // trace_macros!(true);
    let bytes = ser!((b, str_rapira), (&c, crate::byte_rapira) | &a);

    let (b1, c1, a1) = deser!(bytes.as_slice(), str_rapira, crate::byte_rapira | u32);
    // trace_macros!(false);

    assert_eq!(b, b1);
    assert_eq!(c, c1);
    assert_eq!(a, a1);
}
