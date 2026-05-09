use std::io::{Error, ErrorKind};

use crate::resp::{CRLF, RESP_MAX_SIZE, Value};
use anyhow::{Result, bail};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, BufReader};

#[derive(Debug)]
pub struct Decoder<R> {
    buf_bulk: bool,
    reader: BufReader<R>,
}

impl<R: AsyncRead + Unpin> Decoder<R> {
    pub fn new(reader: R) -> Self {
        Self {
            buf_bulk: false,
            reader: BufReader::new(reader),
        }
    }

    pub fn with_buf_bulk(reader: R) -> Self {
        Self {
            buf_bulk: true,
            reader: BufReader::new(reader),
        }
    }

    pub async fn decode(&mut self) -> Result<Value> {
        let mut line: Vec<u8> = Vec::new();
        self.reader
            .read_until(b'\n', &mut line)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        let (&type_byte, _) = line
            .split_first()
            .ok_or_else(|| Error::new(ErrorKind::UnexpectedEof, "unexpected EOF"))?;

        if !line.ends_with(CRLF) {
            bail!("missing CRLF: {:?}", line);
        }
        if line.len() < 3 {
            bail!("line too short: {}", line.len());
        }

        let payload = line
            .get(1..line.len() - 2)
            .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "line too short"))?;

        match type_byte {
            b'_' => Ok(Value::Null),
            b'+' => Ok(Value::String(payload.to_vec().into())),
            b'-' => Ok(Value::Error(payload.to_vec().into())),
            b':' => parse_integer(payload).map(Value::Integer),
            b'#' => match payload {
                b"t" => Ok(Value::Boolean(true)),
                b"f" => Ok(Value::Boolean(false)),
                _ => bail!("invalid boolean payload: {:?}", payload),
            },
            b'$' => {
                let n = parse_integer(payload)?;
                if n == -1 {
                    return Ok(Value::Null);
                }
                if n < 0 || n >= RESP_MAX_SIZE {
                    bail!("invalid bulk length: {}", n);
                }
                #[expect(clippy::cast_sign_loss, reason = "checked n >= 0 above")]
                let n = n as usize;
                let mut buf = vec![0u8; n + 2];
                self.reader
                    .read_exact(&mut buf)
                    .await
                    .map_err(|e| anyhow::anyhow!(e))?;
                if !buf.ends_with(CRLF) {
                    bail!("bulk string missing CRLF: {:?}", buf);
                }
                buf.truncate(n);
                if self.buf_bulk {
                    Ok(Value::BulkString(buf.into()))
                } else {
                    Ok(Value::String(buf.into()))
                }
            }
            b'*' => {
                let n = parse_integer(payload)?;
                if n == -1 {
                    return Ok(Value::NullArray);
                }
                if n < 0 || n >= RESP_MAX_SIZE {
                    bail!("invalid array length: {}", n);
                }
                #[expect(clippy::cast_sign_loss, reason = "checked n >= 0 above")]
                let n = n as usize;
                let mut array = Vec::with_capacity(n);
                for _ in 0..n {
                    array.push(Box::pin(self.decode()).await?);
                }
                Ok(Value::Array(array))
            }
            _ => bail!("invalid RESP type byte: {:#04x}", type_byte),
        }
    }
}

#[inline]
fn parse_integer(bytes: &[u8]) -> Result<i64> {
    let s = std::str::from_utf8(bytes)?;
    Ok(s.parse::<i64>()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn decode(input: &[u8]) -> Value {
        Decoder::new(input).decode().await.expect("decode failed")
    }

    #[tokio::test]
    async fn simple_string() {
        assert_eq!(decode(b"+OK\r\n").await, Value::String("OK".into()));
    }

    #[tokio::test]
    async fn error() {
        assert_eq!(
            decode(b"-ERR unknown command\r\n").await,
            Value::Error("ERR unknown command".into())
        );
    }

    #[tokio::test]
    async fn integer() {
        assert_eq!(decode(b":42\r\n").await, Value::Integer(42));
        assert_eq!(decode(b":-1\r\n").await, Value::Integer(-1));
    }

    #[tokio::test]
    async fn null_bulk() {
        assert_eq!(decode(b"$-1\r\n").await, Value::Null);
    }

    #[tokio::test]
    async fn null_array() {
        assert_eq!(decode(b"*-1\r\n").await, Value::NullArray);
    }

    #[tokio::test]
    async fn null_resp3() {
        assert_eq!(decode(b"_\r\n").await, Value::Null);
    }

    #[tokio::test]
    async fn boolean() {
        assert_eq!(decode(b"#t\r\n").await, Value::Boolean(true));
        assert_eq!(decode(b"#f\r\n").await, Value::Boolean(false));
    }

    #[tokio::test]
    async fn bulk_string() {
        assert_eq!(
            Decoder::with_buf_bulk(b"$5\r\nhello\r\n" as &[u8])
                .decode()
                .await
                .expect("decode failed"),
            Value::BulkString("hello".into())
        );
    }

    #[tokio::test]
    async fn bulk_string_as_string() {
        assert_eq!(
            decode(b"$5\r\nhello\r\n").await,
            Value::String("hello".into())
        );
    }

    #[tokio::test]
    async fn array() {
        assert_eq!(
            decode(b"*2\r\n+foo\r\n:99\r\n").await,
            Value::Array(vec![Value::String("foo".into()), Value::Integer(99)])
        );
    }

    #[tokio::test]
    async fn empty_array() {
        assert_eq!(decode(b"*0\r\n").await, Value::Array(vec![]));
    }
}
