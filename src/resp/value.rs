use crate::{ByteString, resp::CRLF};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Value {
    Null,
    NullArray,
    String(ByteString),
    Error(ByteString),
    Integer(i64),
    BulkString(ByteString),
    Array(Vec<Value>),
    Boolean(bool),
}

impl Value {
    pub fn encode(&self) -> ByteString {
        let mut res: ByteString = ByteString(Vec::new());
        match self {
            Value::Null => {
                res.extend_from_slice(b"_");
                res.extend_from_slice(CRLF);
            }
            Value::NullArray => {
                res.extend_from_slice(b"*-1");
                res.extend_from_slice(CRLF);
            }
            Value::String(val) => {
                res.push(b'+');
                res.extend_from_slice(val.as_bytes());
                res.extend_from_slice(CRLF);
            }
            Value::Error(val) => {
                res.push(b'-');
                res.extend_from_slice(val.as_bytes());
                res.extend_from_slice(CRLF);
            }
            Value::Integer(val) => {
                res.push(b':');
                res.extend_from_slice(val.to_string().as_bytes());
                res.extend_from_slice(CRLF);
            }
            Value::BulkString(val) => {
                res.push(b'$');
                res.extend_from_slice(val.as_bytes().len().to_string().as_bytes());
                res.extend_from_slice(CRLF);
                res.extend_from_slice(val.as_bytes());
                res.extend_from_slice(CRLF);
            }
            Value::Boolean(val) => {
                res.push(b'#');
                res.push(if *val { b't' } else { b'f' });
                res.extend_from_slice(CRLF);
            }
            Value::Array(vals) => {
                res.push(b'*');
                res.extend_from_slice(vals.len().to_string().as_bytes());
                res.extend_from_slice(CRLF);
                for val in vals {
                    res.extend_from_slice(val.encode().as_bytes());
                }
            }
        }
        res
    }

    pub fn to_string_pretty(&self) -> String {
        match self {
            Value::Null => "(nil)".to_string(),
            Value::NullArray => "(null array)".to_string(),
            Value::String(val) => val.to_string(),
            Value::Error(val) => format!("(error) {}", val),
            Value::Integer(val) => format!("(integer) {}", val),
            Value::BulkString(val) => val.to_string(),
            Value::Boolean(val) => format!("({})", val),
            Value::Array(vals) => Self::format_array(vals, 0),
        }
    }

    #[inline]
    fn format_array(vals: &Vec<Value>, indent_level: usize) -> String {
        if vals.is_empty() {
            return "empty array".to_string();
        }

        let mut res = String::new();
        for (idx, val) in vals.iter().enumerate() {
            if idx != 0 {
                for _ in 0..indent_level {
                    res.push('\t');
                }
            }
            res.push_str(" - ");
            match val {
                Value::Array(val) => res.push_str(&Self::format_array(val, indent_level + 1)),
                _ => res.push_str(&val.to_string_pretty()),
            };
            res.push('\n');
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use super::Value;

    #[test]
    fn null_pretty() {
        assert_eq!(Value::Null.to_string_pretty(), "(nil)");
    }

    #[test]
    fn null_array_pretty() {
        assert_eq!(Value::NullArray.to_string_pretty(), "(null array)");
    }

    #[test]
    fn string_pretty() {
        assert_eq!(Value::String("hello".into()).to_string_pretty(), "hello");
    }

    #[test]
    fn error_pretty() {
        assert_eq!(
            Value::Error("ERR unknown command".into()).to_string_pretty(),
            "(error) ERR unknown command"
        );
    }

    #[test]
    fn integer_pretty() {
        assert_eq!(Value::Integer(42).to_string_pretty(), "(integer) 42");
        assert_eq!(Value::Integer(-1).to_string_pretty(), "(integer) -1");
        assert_eq!(Value::Integer(0).to_string_pretty(), "(integer) 0");
    }

    #[test]
    fn bulk_string_pretty() {
        assert_eq!(Value::BulkString("foo".into()).to_string_pretty(), "foo");
    }

    #[test]
    fn boolean_pretty() {
        assert_eq!(Value::Boolean(true).to_string_pretty(), "(true)");
        assert_eq!(Value::Boolean(false).to_string_pretty(), "(false)");
    }

    #[test]
    fn empty_array_pretty() {
        assert_eq!(Value::Array(vec![]).to_string_pretty(), "empty array");
    }

    #[test]
    fn flat_array_pretty() {
        let arr = Value::Array(vec![
            Value::String("a".into()),
            Value::Integer(1),
            Value::Null,
        ]);
        assert_eq!(
            arr.to_string_pretty(),
            " - a
 - (integer) 1
 - (nil)
"
        );
    }

    #[test]
    fn nested_array_pretty() {
        let inner = Value::Array(vec![Value::String("x".into()), Value::String("y".into())]);
        let outer = Value::Array(vec![Value::String("top".into()), inner]);
        assert_eq!(
            outer.to_string_pretty(),
            " - top
 -  - x
	 - y

"
        );
    }
}
