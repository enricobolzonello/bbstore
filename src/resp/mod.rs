mod decode;
mod value;

const CRLF: &[u8] = b"\r\n";
const RESP_MAX_SIZE: i64 = 512 * 1024 * 1024;

pub use decode::Decoder;
pub use value::Value;
