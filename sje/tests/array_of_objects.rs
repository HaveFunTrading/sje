use sje_derive::Decoder;

#[derive(Decoder)]
#[sje(object)]
#[allow(dead_code)]
struct Position {
    #[sje(rename = "s")]
    symbol: String,
    #[sje(rename = "a")]
    amount: u32,
}

// #[derive(Decoder)]
// #[sje(object)]
// #[allow(dead_code)]
// struct PositionUpdate {
//     #[sje(rename = "t")]
//     timestamp: u64,
//     #[sje(rename = "u")]
//     updates: Vec<Position>,
// }

// implies we have *Decoder, has to be explicit, i.e. decoder = true, otherwise it will try to use from_str

mod wtf {
    use crate::PositionDecoder;
    use std::slice::from_raw_parts;

    #[derive(Debug)]
    pub struct PositionUpdateDecoder<'a> {
        timestamp: sje::LazyField<'a, u64>,
        updates: (&'a [u8], usize),
    }
    impl<'a> PositionUpdateDecoder<'a> {
        #[inline]
        pub fn decode(bytes: &'a [u8]) -> Result<Self, sje::error::Error> {
            let mut scanner = sje::scanner::JsonScanner::wrap(bytes);
            scanner.skip(5usize);
            let (offset, len) = scanner
                .next_number()
                .ok_or_else(|| sje::error::Error::MissingField("timestamp"))?;
            let timestamp = sje::LazyField::from_bytes(unsafe { bytes.get_unchecked(offset..offset + len) });
            scanner.skip(5usize);
            let (offset, len, count) = scanner
                .next_array()
                .ok_or_else(|| sje::error::Error::MissingField("updates"))?;
            let updates = (unsafe { bytes.get_unchecked(offset..offset + len) }, count);
            Ok(Self { timestamp, updates })
        }
    }
    impl<'a> PositionUpdateDecoder<'a> {
        #[inline]
        pub const fn timestamp_as_slice(&self) -> &[u8] {
            self.timestamp.as_slice()
        }
        #[inline]
        pub const fn timestamp_as_str(&self) -> &str {
            self.timestamp.as_str()
        }
        #[inline]
        pub const fn timestamp_as_lazy_field(&self) -> &sje::LazyField<'a, u64> {
            &self.timestamp
        }
        #[inline]
        pub const fn updates_as_slice(&self) -> &[u8] {
            self.updates.0
        }
        #[inline]
        pub const fn updates_as_str(&self) -> &str {
            unsafe { std::str::from_utf8_unchecked(self.updates_as_slice()) }
        }
        #[inline]
        pub const fn updates_count(&self) -> usize {
            self.updates.1
        }
    }
    impl PositionUpdateDecoder<'_> {
        #[inline]
        pub fn timestamp(&self) -> u64 {
            self.timestamp.get().unwrap()
        }
    }
    #[derive(Debug)]
    pub struct Updates<'a> {
        bytes: &'a [u8],
        remaining: usize,
    }
    impl PositionUpdateDecoder<'_> {
        #[inline]
        pub const fn updates(&self) -> Updates {
            Updates {
                bytes: self.updates.0,
                remaining: self.updates.1,
            }
        }
    }

    impl<'a> IntoIterator for Updates<'a> {
        type Item = PositionDecoder<'a>;
        type IntoIter = UpdatesIter<'a>;
        fn into_iter(self) -> Self::IntoIter {
            UpdatesIter {
                scanner: sje::scanner::JsonScanner::wrap(self.bytes),
                remaining: self.remaining,
            }
        }
    }
    pub struct UpdatesIter<'a> {
        scanner: sje::scanner::JsonScanner<'a>,
        remaining: usize,
    }
    impl<'a> Iterator for UpdatesIter<'a> {
        type Item = PositionDecoder<'a>;
        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            if self.scanner.position() + 1 == self.scanner.bytes().len() { return None; }
            self.scanner.skip(1);
            let (offset, len) = self.scanner.next_object().unwrap();
            self.remaining -= 1;
            let bytes = &self.scanner.bytes()[offset..offset + len];
            let bytes = unsafe { from_raw_parts(bytes.as_ptr(), bytes.len()) };
            Some(PositionDecoder::decode(bytes).unwrap())
        }
        #[inline]
        fn size_hint(&self) -> (usize, Option<usize>) {
            (self.remaining, Some(self.remaining))
        }
    }
    impl ExactSizeIterator for UpdatesIter<'_> {
        #[inline]
        fn len(&self) -> usize {
            self.remaining
        }
    }

    #[test]
    fn test() {
        let json = r#"{"t":12345,"u":[{"s":"btcusdt","a":100},{"s":"ethusdt","a":200}]}"#;

        let update = PositionUpdateDecoder::decode(json.as_bytes()).unwrap();
        assert_eq!(2, update.updates_count());
        
        let mut positions = update.updates().into_iter();

        let position = positions.next().unwrap();
        assert_eq!("btcusdt", position.symbol_as_str());
        assert_eq!(100, position.amount());

        let position = positions.next().unwrap();
        assert_eq!("ethusdt", position.symbol_as_str());
        assert_eq!(200, position.amount());
    
        assert!(positions.next().is_none());
    }
}
