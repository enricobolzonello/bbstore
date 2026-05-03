use std::{fmt, ops::Deref};

#[derive(Hash)]
pub(crate) struct ByteStr(pub [u8]);

impl ByteStr {
    #[inline]
    pub fn new<B: ?Sized + AsRef<[u8]>>(bytes: &B) -> &Self {
        ByteStr::from_bytes(bytes.as_ref())
    }

    #[inline]
    pub const fn from_bytes(slice: &[u8]) -> &Self {
        // SAFETY: `ByteStr` is a transparent wrapper around `[u8]`, so we can turn a reference to
        // the wrapped type into a reference to the wrapper type.
        unsafe { &*(slice as *const [u8] as *const Self) }
    }
}

#[derive(PartialEq, Debug, Hash, Eq, Clone)]
pub(crate) struct ByteString(pub Vec<u8>);

impl ByteString {
    #[inline]
    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl From<Vec<u8>> for ByteString {
    #[inline]
    fn from(value: Vec<u8>) -> Self {
        ByteString(value)
    }
}

impl From<String> for ByteString {
    #[inline]
    fn from(value: String) -> Self {
        ByteString(value.into_bytes())
    }
}

impl From<&str> for ByteString {
    #[inline]
    fn from(value: &str) -> Self {
        ByteString(value.as_bytes().to_vec())
    }
}

impl AsRef<ByteStr> for ByteString {
    #[inline]
    fn as_ref(&self) -> &ByteStr {
        ByteStr::new(&self.0)
    }
}

impl Deref for ByteString {
    type Target = ByteStr;

    fn deref(&self) -> &Self::Target {
        ByteStr::from_bytes(&self.0)
    }
}

impl fmt::Display for ByteString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.0))
    }
}
