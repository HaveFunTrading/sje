#[macro_export]
macro_rules! field_impl1 {
    ($method_name:ident, $method_name_with_known_len:ident $quoted:expr, $one:literal) => {
        impl<'a> JsonScanner<'a> {
            #[inline]
            pub fn $method_name(&mut self) -> Option<(usize, usize)> {
                let offset = self.cursor + $quoted;
                let len = memchr::memchr($one, unsafe { self.bytes.get_unchecked(offset..) })?;
                self.cursor += len + $quoted * 2;
                Some((offset, len))
            }

            #[inline]
            pub const fn $method_name_with_known_len(&mut self, len: usize) -> Option<(usize, usize)> {
                let offset = self.cursor + $quoted;
                self.cursor += len + $quoted * 2;
                Some((offset, len))
            }
        }
    };
}

#[macro_export]
macro_rules! field_impl3 {
    ($method_name:ident, $method_name_with_known_len:ident, $quoted:expr, $one:literal, $two:literal, $three:literal) => {
        impl<'a> JsonScanner<'a> {
            #[inline]
            pub fn $method_name(&mut self) -> Option<(usize, usize)> {
                let offset = self.cursor + $quoted;
                let len = memchr::memchr3($one, $two, $three, unsafe { self.bytes.get_unchecked(offset..) })?;
                self.cursor += len + $quoted * 2;
                Some((offset, len))
            }

            #[inline]
            pub const fn $method_name_with_known_len(&mut self, len: usize) -> Option<(usize, usize)> {
                let offset = self.cursor + $quoted;
                self.cursor += len + $quoted * 2;
                Some((offset, len))
            }
        }
    };
}

#[macro_export]
macro_rules! composite_impl {
    ($method_name:ident, $open_char:literal, $close_char:literal) => {
        impl<'a> JsonScanner<'a> {
            #[inline]
            pub const fn $method_name(&mut self) -> Option<(usize, usize)> {
                let bytes = self.bytes;
                let start = self.cursor;
                let mut counter: u32 = 1;
                let mut i: usize = 0;
        
                loop {
                    // if we've run off the end, give up
                    let idx = start + 1 + i;
                    if idx >= bytes.len() {
                        return None;
                    }
        
                    // fetch the next byte after the opening char
                    let b = bytes[idx];
        
                    // bump the nesting counter
                    match b {
                        $open_char => counter += 1,
                        $close_char => counter -= 1,
                        _    => {}
                    }
        
                    // if we've closed the top‐level object, return its span
                    if counter == 0 {
                        self.cursor = start + i + 2;
                        return Some((start, i + 2));
                    }
        
                    i += 1;
                }
            }
        }
    };
}


// #[macro_export]
// macro_rules! composite_impl {
//     ($method_name:ident, $open_char:literal, $close_char:literal) => {
//         impl<'a> JsonScanner<'a> {
//             #[inline]
//             pub fn $method_name(&mut self) -> Option<(usize, usize)> {
//                 let offset = self.cursor;
//                 let mut counter = 1u32;
//                 for (index, &item) in unsafe { self.bytes.get_unchecked(offset + 1..) }
//                     .iter()
//                     .enumerate()
//                 {
//                     match item {
//                         $open_char => counter += 1,
//                         $close_char => counter -= 1,
//                         _ => {}
//                     }
//                     if counter == 0 {
//                         self.cursor += index + 2;
//                         return Some((offset, index + 2));
//                     }
//                 }
//                 None
//             }
//         }
//     };
// }
