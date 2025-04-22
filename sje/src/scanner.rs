use crate::{composite_impl, field_impl1, field_impl3};

pub struct JsonScanner<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

impl<'a> JsonScanner<'a> {
    #[inline]
    pub const fn wrap(bytes: &'a [u8]) -> Self {
        Self { bytes, cursor: 0 }
    }

    #[inline]
    pub const fn skip(&mut self, count: usize) {
        self.cursor += count;
    }

    #[inline]
    pub const fn position(&self) -> usize {
        self.cursor
    }

    #[inline]
    pub const fn bytes(&self) -> &[u8] {
        self.bytes
    }
}

field_impl1!(next_string, next_string_with_known_len 1, b'"');
field_impl3!(next_number, next_number_with_known_len, 0, b',', b']', b'}');
field_impl3!(next_boolean, next_boolean_with_known_len, 0, b',', b']', b'}');
composite_impl!(next_tuple, b'[', b']');
composite_impl!(next_object, b'{', b'}');

impl JsonScanner<'_> {
    #[inline]
    pub fn next_array(&mut self) -> Option<(usize, usize, usize)> {
        let offset = self.cursor;
        let mut counter = 1u32;
        let mut array_len = 0;
        for (index, &item) in unsafe { self.bytes.get_unchecked(offset + 1..) }.iter().enumerate() {
            match item {
                b'[' => counter += 1,
                b']' => counter -= 1,
                _ => {}
            }

            if item == b',' && counter == 1 {
                array_len += 1;
            }

            if counter == 0 {
                if index > 0 {
                    let previous = unsafe { *self.bytes.get_unchecked(index - 1) } as char;
                    if previous != '[' {
                        array_len += 1;
                    }
                }
                self.cursor += index + 2;
                return Some((offset, index + 2, array_len));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::scanner::JsonScanner;

    #[test]
    fn should_scan_strings_and_numbers() {
        let bytes = br#"{"e":"depthUpdate","E":1704907109810,"s":"BTCUSDT","U":41933235159,"u":41933235172}"#;
        let mut scanner = JsonScanner::wrap(bytes);

        scanner.skip(5);
        let (offset, len) = scanner.next_string().unwrap();
        assert_eq!("depthUpdate".as_bytes(), &bytes[offset..offset + len]);

        scanner.skip(5);
        let (offset, len) = scanner.next_number().unwrap();
        assert_eq!("1704907109810".as_bytes(), &bytes[offset..offset + len]);

        scanner.skip(5);
        let (offset, len) = scanner.next_string().unwrap();
        assert_eq!("BTCUSDT".as_bytes(), &bytes[offset..offset + len]);

        scanner.skip(5);
        let (offset, len) = scanner.next_number().unwrap();
        assert_eq!("41933235159".as_bytes(), &bytes[offset..offset + len]);

        scanner.skip(5);
        let (offset, len) = scanner.next_number().unwrap();
        assert_eq!("41933235172".as_bytes(), &bytes[offset..offset + len]);
    }

    #[test]
    fn should_scan_only_strings() {
        let bytes = br#"{"a":"foo","b":"bar","c":"baz"}"#;
        let mut scanner = JsonScanner::wrap(bytes);

        scanner.skip(5);
        let (offset, len) = scanner.next_string().unwrap();
        assert_eq!("foo".as_bytes(), &bytes[offset..offset + len]);

        scanner.skip(5);
        let (offset, len) = scanner.next_string().unwrap();
        assert_eq!("bar".as_bytes(), &bytes[offset..offset + len]);

        scanner.skip(5);
        let (offset, len) = scanner.next_string().unwrap();
        assert_eq!("baz".as_bytes(), &bytes[offset..offset + len]);
    }

    #[test]
    fn should_scan_array() {
        let bytes = br#"{"b":[1,2,3],"a":[4,5],"E":1704907109810,"c":[[5,6,7],[8,9]],"d":[],"e":[2]}"#;
        let mut scanner = JsonScanner::wrap(bytes);

        scanner.skip(5);
        let (offset, len, count) = scanner.next_array().unwrap();
        assert_eq!("[1,2,3]".as_bytes(), &bytes[offset..offset + len]);
        assert_eq!(3, count);

        scanner.skip(5);
        let (offset, len, count) = scanner.next_array().unwrap();
        assert_eq!("[4,5]".as_bytes(), &bytes[offset..offset + len]);
        assert_eq!(2, count);

        scanner.skip(5);
        let (offset, len) = scanner.next_number().unwrap();
        assert_eq!("1704907109810".as_bytes(), &bytes[offset..offset + len]);

        scanner.skip(5);
        let (offset, len, count) = scanner.next_array().unwrap();
        assert_eq!("[[5,6,7],[8,9]]".as_bytes(), &bytes[offset..offset + len]);
        assert_eq!(2, count);

        scanner.skip(5);
        let (offset, len, count) = scanner.next_array().unwrap();
        assert_eq!("[]".as_bytes(), &bytes[offset..offset + len]);
        assert_eq!(0, count);

        scanner.skip(5);
        let (offset, len, count) = scanner.next_array().unwrap();
        assert_eq!("[2]".as_bytes(), &bytes[offset..offset + len]);
        assert_eq!(1, count);
    }

    #[test]
    fn should_scan_array_elements() {
        let bytes = br#"[1,200,30]"#;
        let mut scanner = JsonScanner::wrap(bytes);

        scanner.skip(1);
        let (offset, len) = scanner.next_number().unwrap();
        assert_eq!("1".as_bytes(), &bytes[offset..offset + len]);

        scanner.skip(1);
        let (offset, len) = scanner.next_number().unwrap();
        assert_eq!("200".as_bytes(), &bytes[offset..offset + len]);

        scanner.skip(1);
        let (offset, len) = scanner.next_number().unwrap();
        assert_eq!("30".as_bytes(), &bytes[offset..offset + len]);
    }

    #[test]
    fn should_scan_empty_array() {
        let bytes = br#"{"b":[],"a":[[]],"c":[[[]]]}"#;
        let mut scanner = JsonScanner::wrap(bytes);

        scanner.skip(5);
        let (offset, len, count) = scanner.next_array().unwrap();
        assert_eq!("[]".as_bytes(), &bytes[offset..offset + len]);
        assert_eq!(0, count);

        scanner.skip(5);
        let (offset, len, count) = scanner.next_array().unwrap();
        assert_eq!("[[]]".as_bytes(), &bytes[offset..offset + len]);
        assert_eq!(1, count);

        scanner.skip(5);
        let (offset, len, count) = scanner.next_array().unwrap();
        assert_eq!("[[[]]]".as_bytes(), &bytes[offset..offset + len]);
        assert_eq!(1, count);
    }

    #[test]
    fn should_scan_object() {
        let bytes = br#"{"b":{"id":1},"a":[4,5],"E":1704907109810,"c":{"id":1,"foo":{"id":2}}}"#;
        let mut scanner = JsonScanner::wrap(bytes);

        scanner.skip(5);
        let (offset, len) = scanner.next_object().unwrap();
        assert_eq!(r#"{"id":1}"#.as_bytes(), &bytes[offset..offset + len]);

        scanner.skip(5);
        let (offset, len, count) = scanner.next_array().unwrap();
        assert_eq!("[4,5]".as_bytes(), &bytes[offset..offset + len]);
        assert_eq!(2, count);

        scanner.skip(5);
        let (offset, len) = scanner.next_number().unwrap();
        assert_eq!("1704907109810".as_bytes(), &bytes[offset..offset + len]);

        scanner.skip(5);
        let (offset, len) = scanner.next_object().unwrap();
        assert_eq!(r#"{"id":1,"foo":{"id":2}}"#.as_bytes(), &bytes[offset..offset + len]);
    }

    #[test]
    fn should_scan_empty_object() {
        let bytes = br#"{"b":{},"c":{"id":{}}}"#;
        let mut scanner = JsonScanner::wrap(bytes);

        scanner.skip(5);
        let (offset, len) = scanner.next_object().unwrap();
        assert_eq!(r#"{}"#.as_bytes(), &bytes[offset..offset + len]);

        scanner.skip(5);
        let (offset, len) = scanner.next_object().unwrap();
        assert_eq!(r#"{"id":{}}"#.as_bytes(), &bytes[offset..offset + len]);
    }

    #[test]
    fn should_scan_boolean() {
        let bytes = br#"{"b":false,"c":true}}"#;
        let mut scanner = JsonScanner::wrap(bytes);

        scanner.skip(5);
        let (offset, len) = scanner.next_boolean().unwrap();
        assert_eq!("false".as_bytes(), &bytes[offset..offset + len]);

        scanner.skip(5);
        let (offset, len) = scanner.next_boolean().unwrap();
        assert_eq!("true".as_bytes(), &bytes[offset..offset + len]);
    }

    #[test]
    fn should_scan_numbers() {
        let bytes = br#"{"a":-1,"b":12.4,"c":-541.56}}"#;
        let mut scanner = JsonScanner::wrap(bytes);

        scanner.skip(5);
        let (offset, len) = scanner.next_number().unwrap();
        assert_eq!("-1".as_bytes(), &bytes[offset..offset + len]);

        scanner.skip(5);
        let (offset, len) = scanner.next_number().unwrap();
        assert_eq!("12.4".as_bytes(), &bytes[offset..offset + len]);

        scanner.skip(5);
        let (offset, len) = scanner.next_number().unwrap();
        assert_eq!("-541.56".as_bytes(), &bytes[offset..offset + len]);
    }

    mod decoder {
        use std::str::from_utf8;

        use crate::scanner::JsonScanner;

        struct L2UpdateDecoder<'a> {
            event: &'a [u8],
            event_time: &'a [u8],
        }

        impl<'a> L2UpdateDecoder<'a> {
            pub fn decode(bytes: &'a [u8]) -> L2UpdateDecoder<'a> {
                let mut scanner = JsonScanner::wrap(bytes);

                scanner.skip(5);
                let (offset, len) = scanner.next_string().unwrap();
                let event = unsafe { bytes.get_unchecked(offset..offset + len) };

                scanner.skip(5);
                let (offset, len) = scanner.next_number().unwrap();
                let event_time = unsafe { bytes.get_unchecked(offset..offset + len) };

                Self { event, event_time }
            }
        }

        #[test]
        fn should_decode() {
            let l2_update = L2UpdateDecoder::decode(
                br#"{"e":"depthUpdate","E":1704907109810,"s":"BTCUSDT","U":41933235159,"u":41933235172}"#,
            );

            assert_eq!("depthUpdate", from_utf8(l2_update.event).unwrap());
            assert_eq!("1704907109810", from_utf8(l2_update.event_time).unwrap())
        }
    }
}
