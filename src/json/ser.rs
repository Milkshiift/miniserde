use crate::json::{Array, Number, Object, Value};
use crate::ser::{Fragment, Map, Seq, Serialize};
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

mod writer {
    use alloc::string::String;
    use alloc::vec::Vec;

    pub trait Write {
        fn write_str(&mut self, s: &str);
        fn write_char(&mut self, c: char);
    }

    impl Write for String {
        #[inline]
        fn write_str(&mut self, s: &str) {
            self.push_str(s);
        }
        #[inline]
        fn write_char(&mut self, c: char) {
            self.push(c);
        }
    }

    impl Write for Vec<u8> {
        #[inline]
        fn write_str(&mut self, s: &str) {
            self.extend_from_slice(s.as_bytes());
        }
        #[inline]
        fn write_char(&mut self, c: char) {
            let mut buf = [0u8; 4];
            self.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
        }
    }
}

/// Convert any serializable type into a `miniserde::json::Value`.
///
/// ```rust
/// use miniserde::{json, Serialize};
/// use miniserde::json::Value;
///
/// #[derive(Serialize)]
/// struct Example {
///     code: u32,
///     message: String,
/// }
///
/// fn main() {
///     let example = Example {
///         code: 200,
///         message: "reminiscent of Serde".to_owned(),
///     };
///
///     let value: Value = json::to_value(&example);
///     println!("{:?}", value);
/// }
/// ```
pub fn to_value<T>(value: &T) -> Value
where
    T: ?Sized + Serialize,
{
    let mut stack = Vec::new();
    let mut fragment = value.begin();

    enum Layer<'a> {
        Seq(Box<dyn Seq + 'a>, Array),
        Map(Box<dyn Map + 'a>, Object, Option<String>),
    }

    loop {
        let val = match fragment {
            Fragment::Null => Value::Null,
            Fragment::Bool(b) => Value::Bool(b),
            Fragment::Str(s) => Value::String(s.into_owned()),
            Fragment::U64(n) => Value::Number(Number::U64(n)),
            Fragment::I64(n) => Value::Number(Number::I64(n)),
            Fragment::F64(n) => Value::Number(Number::F64(n)),
            Fragment::Seq(mut seq) => {
                let next = unsafe { extend_lifetime!(seq.next() as Option<&dyn Serialize>) };
                match next {
                    Some(first) => {
                        stack.push(Layer::Seq(seq, Array::new()));
                        fragment = first.begin();
                        continue;
                    }
                    None => Value::Array(Array::new()),
                }
            }
            Fragment::Map(mut map) => {
                let next = unsafe {
                    extend_lifetime!(map.next() as Option<(Cow<str>, &dyn Serialize)>)
                };
                match next {
                    Some((key, first)) => {
                        stack.push(Layer::Map(map, Object::new(), Some(key.into_owned())));
                        fragment = first.begin();
                        continue;
                    }
                    None => Value::Object(Object::new()),
                }
            }
        };

        let mut current_val = val;
        loop {
            match stack.last_mut() {
                None => return current_val,
                Some(Layer::Seq(seq, arr)) => {
                    arr.push(current_val);
                    let next = unsafe { extend_lifetime!(seq.next() as Option<&dyn Serialize>) };
                    match next {
                        Some(next_elem) => {
                            fragment = next_elem.begin();
                            break;
                        }
                        None => {
                            let arr = match stack.pop() {
                                Some(Layer::Seq(_, a)) => a,
                                _ => unreachable!(),
                            };
                            current_val = Value::Array(arr);
                        }
                    }
                }
                Some(Layer::Map(map, obj, key_opt)) => {
                    let key = key_opt.take().expect("Map layer without pending key");
                    obj.insert(key, current_val);
                    let next = unsafe {
                        extend_lifetime!(map.next() as Option<(Cow<str>, &dyn Serialize)>)
                    };
                    match next {
                        Some((key, next_elem)) => {
                            *key_opt = Some(key.into_owned());
                            fragment = next_elem.begin();
                            break;
                        }
                        None => {
                            let obj = match stack.pop() {
                                Some(Layer::Map(_, o, _)) => o,
                                _ => unreachable!(),
                            };
                            current_val = Value::Object(obj);
                        }
                    }
                }
            }
        }
    }
}

/// Serialize any serializable type into a JSON string.
///
/// ```rust
/// use miniserde::{json, Serialize};
///
/// #[derive(Serialize, Debug)]
/// struct Example {
///     code: u32,
///     message: String,
/// }
///
/// fn main() {
///     let example = Example {
///         code: 200,
///         message: "reminiscent of Serde".to_owned(),
///     };
///
///     let j = json::to_string(&example);
///     println!("{}", j);
/// }
/// ```
pub fn to_string<T>(value: &T) -> String
where
    T: ?Sized + Serialize,
{
    let mut out = String::with_capacity(128);
    to_writer_impl(&value, &mut out);
    out
}

pub fn to_vec<T>(value: &T) -> Vec<u8>
where
    T: ?Sized + Serialize,
{
    let mut out = Vec::with_capacity(128);
    to_writer_impl(&value, &mut out);
    out
}

struct Serializer<'a> {
    stack: Vec<Layer<'a>>,
}

enum Layer<'a> {
    Seq(Box<dyn Seq + 'a>),
    Map(Box<dyn Map + 'a>),
}

fn to_writer_impl<W>(value: &dyn Serialize, out: &mut W)
where
    W: ?Sized + writer::Write,
{
    let mut serializer = Serializer { stack: Vec::new() };
    let mut fragment = value.begin();

    'outer: loop {
        match fragment {
            Fragment::Null => out.write_str("null"),
            Fragment::Bool(b) => out.write_str(if b { "true" } else { "false" }),
            Fragment::Str(s) => escape_str(&s, out),
            Fragment::U64(n) => out.write_str(itoa::Buffer::new().format(n)),
            Fragment::I64(n) => out.write_str(itoa::Buffer::new().format(n)),
            Fragment::F64(n) => {
                if n.is_finite() {
                    out.write_str(ryu::Buffer::new().format_finite(n));
                } else {
                    out.write_str("null");
                }
            }
            Fragment::Seq(mut seq) => {
                out.write_char('[');
                // invariant: `seq` must outlive `first`
                match unsafe { extend_lifetime!(seq.next() as Option<&dyn Serialize>) } {
                    Some(first) => {
                        serializer.stack.push(Layer::Seq(seq));
                        fragment = first.begin();
                        continue 'outer;
                    }
                    None => out.write_char(']'),
                }
            }
            Fragment::Map(mut map) => {
                out.write_char('{');
                // invariant: `map` must outlive `first`
                match unsafe { extend_lifetime!(map.next() as Option<(Cow<str>, &dyn Serialize)>) }
                {
                    Some((key, first)) => {
                        escape_str(&key, out);
                        out.write_char(':');
                        serializer.stack.push(Layer::Map(map));
                        fragment = first.begin();
                        continue 'outer;
                    }
                    None => out.write_char('}'),
                }
            }
        }

        loop {
            match serializer.stack.last_mut() {
                Some(Layer::Seq(seq)) => {
                    // invariant: `seq` must outlive `next`
                    match unsafe { extend_lifetime!(seq.next() as Option<&dyn Serialize>) } {
                        Some(next) => {
                            out.write_char(',');
                            fragment = next.begin();
                            break;
                        }
                        None => {
                            out.write_char(']');
                            serializer.stack.pop();
                        }
                    }
                }
                Some(Layer::Map(map)) => {
                    // invariant: `map` must outlive `next`
                    match unsafe {
                        extend_lifetime!(map.next() as Option<(Cow<str>, &dyn Serialize)>)
                    } {
                        Some((key, next)) => {
                            out.write_char(',');
                            escape_str(&key, out);
                            out.write_char(':');
                            fragment = next.begin();
                            break;
                        }
                        None => {
                            out.write_char('}');
                            serializer.stack.pop();
                        }
                    }
                }
                None => return,
            }
        }
    }
}

fn escape_str<W>(value: &str, out: &mut W)
where
    W: ?Sized + writer::Write,
{
    out.write_char('"');

    let mut start = 0;
    let bytes = value.as_bytes();

    for (i, &byte) in bytes.iter().enumerate() {
        let escape = ESCAPE[byte as usize];
        if escape == 0 {
            continue;
        }

        if start < i {
            out.write_str(unsafe { core::str::from_utf8_unchecked(&bytes[start..i]) });
        }

        let escaped_char = match escape {
            BB => "\\b",
            TT => "\\t",
            NN => "\\n",
            FF => "\\f",
            RR => "\\r",
            QU => "\\\"",
            BS => "\\\\",
            U => {
                static HEX_DIGITS: [u8; 16] = *b"0123456789abcdef";
                let mut buf = [0u8; 6];
                buf[0] = b'\\';
                buf[1] = b'u';
                buf[2] = b'0';
                buf[3] = b'0';
                buf[4] = HEX_DIGITS[(byte >> 4) as usize];
                buf[5] = HEX_DIGITS[(byte & 0xF) as usize];

                out.write_str(unsafe { core::str::from_utf8_unchecked(&buf) });
                start = i + 1;
                continue;
            }
            _ => unreachable!(),
        };
        out.write_str(escaped_char);

        start = i + 1;
    }

    if start < bytes.len() {
        out.write_str(unsafe { core::str::from_utf8_unchecked(&bytes[start..]) });
    }

    out.write_char('"');
}

const BB: u8 = b'b'; // \x08
const TT: u8 = b't'; // \x09
const NN: u8 = b'n'; // \x0A
const FF: u8 = b'f'; // \x0C
const RR: u8 = b'r'; // \x0D
const QU: u8 = b'"'; // \x22
const BS: u8 = b'\\'; // \x5C
const U: u8 = b'u'; // \x00...\x1F except the ones above

// Lookup table of escape sequences. A value of b'x' at index i means that byte
// i is escaped as "\x" in JSON. A value of 0 means that byte i is not escaped.
#[rustfmt::skip]
static ESCAPE: [u8; 256] = [
    //  1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
    U,  U,  U,  U,  U,  U,  U,  U, BB, TT, NN,  U, FF, RR,  U,  U, // 0
    U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U, // 1
    0,  0, QU,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 2
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 3
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 4
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, BS,  0,  0,  0, // 5
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 6
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 7
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 8
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 9
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // A
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // B
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // C
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // D
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // E
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // F
];