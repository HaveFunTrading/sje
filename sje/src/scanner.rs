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
    pub const fn next_array(&mut self) -> Option<(usize, usize, usize)> {
        let bytes = self.bytes;
        let start = self.cursor;

        // state
        let mut array_depth = 0usize;
        let mut obj_depth = 0usize;
        let mut in_string = false;
        let mut escaped = false;
        let mut commas = 0usize;
        let mut saw_value = false;
        let mut index = 0usize;

        // iterate until we run off the end
        loop {
            // bounds-check
            if start + index >= bytes.len() {
                return None;
            }
            let b = bytes[start + index];

            // 1) handle strings & escapes
            if in_string {
                if escaped {
                    escaped = false;
                } else if b == b'\\' {
                    escaped = true;
                } else if b == b'"' {
                    in_string = false;
                }
                index += 1;
                continue;
            } else if b == b'"' {
                in_string = true;
                index += 1;
                continue;
            }

            // 2) track nesting and detect top-level elements
            match b {
                // entering any array
                b'[' => {
                    array_depth += 1;
                    if array_depth == 1 {
                        // this is the '[' of *our* array
                        saw_value = false;
                    } else if array_depth == 2 {
                        // nested array => counts as an element
                        saw_value = true;
                    }
                }

                // leaving an array
                b']' => {
                    array_depth -= 1;
                    if array_depth == 0 {
                        // done with this array
                        let element_count = if saw_value { commas + 1 } else { 0 };
                        // advance cursor past the closing ]
                        self.cursor = start + index + 1;
                        return Some((start, index + 1, element_count));
                    }
                }

                // entering an object (only matters inside our array)
                b'{' if array_depth > 0 => {
                    if array_depth == 1 && obj_depth == 0 {
                        // top-level object => counts as element
                        saw_value = true;
                    }
                    obj_depth += 1;
                }

                // leaving an object
                b'}' if array_depth > 0 => {
                    obj_depth -= 1;
                }

                // a comma that really separates two top-level elements
                b',' if array_depth == 1 && obj_depth == 0 => {
                    commas += 1;
                    saw_value = false; // now look for next element
                }

                // any other non-whitespace at top-level marks “we saw a value”
                _ if array_depth == 1 && !b.is_ascii_whitespace() => {
                    saw_value = true;
                }

                _ => {}
            }

            index += 1;
        }
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
    fn should_scan_bool_array() {
        let bytes = br#"[true,false,false]"#;
        let mut scanner = JsonScanner::wrap(bytes);

        let (_, _, count) = scanner.next_array().unwrap();
        assert_eq!(3, count)
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

    #[test]
    fn should_scan_array_of_objects() {
        let bytes = br#"[{"s":"btcusdt","a":100},{"s":"ethusdt","a":200}]"#;
        let mut scanner = JsonScanner::wrap(bytes);
        scanner.skip(0);
        let (offset, len, count) = scanner.next_array().unwrap();

        assert_eq!(2, count)
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
