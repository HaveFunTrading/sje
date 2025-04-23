use crate::error::Error;
use std::cell::UnsafeCell;
use std::str::{from_utf8_unchecked, FromStr};

pub mod error;
mod macros;
pub mod scanner;

#[cfg(feature = "derive")]
pub use sje_derive::Decoder;

#[derive(Debug)]
pub struct LazyField<'a, T> {
    inner: UnsafeCell<Field<'a, T>>,
}

#[derive(Debug)]
enum Field<'a, T> {
    Bytes(&'a [u8]),
    Parsed(&'a [u8], T),
}

impl<'a, T> From<&'a [u8]> for LazyField<'a, T> {
    #[inline]
    fn from(s: &'a [u8]) -> Self {
        Self {
            inner: UnsafeCell::new(Field::Bytes(s)),
        }
    }
}

impl<T: Clone + FromStr> LazyField<'_, T> {
    #[inline]
    pub fn get(&self) -> Result<T, Error> {
        Ok((*self.get_ref()?).clone())
    }
}

impl<T: FromStr> LazyField<'_, T> {
    #[inline]
    pub fn get_ref(&self) -> Result<&T, Error> {
        // SAFETY: We use UnsafeCell to gain mutable access.
        // It is up to you to ensure that this mutation is safe (e.g., no concurrent
        // accesses) and that T's invariants are upheld.
        unsafe {
            let field = &mut *self.inner.get();
            match field {
                Field::Bytes(bytes) => {
                    let s = from_utf8_unchecked(bytes);
                    let parsed = s.parse().map_err(|_| Error::Parse(s.to_owned()))?;
                    *field = Field::Parsed(bytes, parsed);
                    match field {
                        Field::Parsed(_, parsed) => Ok(parsed),
                        _ => unreachable!(),
                    }
                }
                Field::Parsed(_, parsed) => Ok(unlikely(parsed)),
            }
        }
    }
}

impl<'a, T> LazyField<'a, T> {
    #[inline]
    pub const fn from_bytes(bytes: &'a [u8]) -> Self {
        Self {
            inner: UnsafeCell::new(Field::Bytes(bytes)),
        }
    }

    #[inline]
    pub const fn as_slice(&self) -> &[u8] {
        // SAFETY: We use UnsafeCell to gain mutable access.
        // It is up to you to ensure that this mutation is safe (e.g., no concurrent
        // accesses) and that T's invariants are upheld.
        unsafe {
            let field = &*self.inner.get();
            match field {
                Field::Bytes(bytes) => bytes,
                Field::Parsed(bytes, _) => unlikely(bytes),
            }
        }
    }

    #[inline]
    pub const fn as_str(&self) -> &str {
        unsafe { from_utf8_unchecked(self.as_slice()) }
    }
}

#[cold]
const fn unlikely<T>(t: T) -> T {
    t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_lazily() {
        #[derive(Copy, Clone, Eq, PartialEq, Debug)]
        struct Price(u64);

        impl FromStr for Price {
            type Err = ();

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(s.parse().map_err(|_| ())?))
            }
        }

        let price = LazyField::<Price>::from_bytes("123".as_bytes());

        assert_eq!("123", price.as_str());
        assert_eq!(b"123", price.as_slice());
        assert_eq!(Price(123), price.get().unwrap());
        assert_eq!(&Price(123), price.get_ref().unwrap());
    }
}
