mod glibc_impls;
pub(crate) mod matcher;

use rsonpath_syntax::str::JsonString;
use std::fmt::Debug;

use crate::{BLOCK_SIZE, JSON_SPACE_BYTE};

#[derive(Clone)]
pub struct StringPattern {
    bytes: Vec<u8>,
    alternatives: Vec<AlternativeRepresentation>,
    len: usize,
    len_limit: usize,
}

impl std::hash::Hash for StringPattern {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.bytes.hash(state);
    }
}

impl PartialOrd for StringPattern {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.bytes.cmp(&other.bytes))
    }
}

impl Ord for StringPattern {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.bytes.cmp(&other.bytes)
    }
}

impl PartialEq for StringPattern {
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
    }
}

impl Eq for StringPattern {}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AlternativeRepresentation {
    SlashUSingle(u32, u8),
    SlashUPair(u32, u32, u8),
    USingle(u32),
    SlashByteOrUSingle(u8, u32),
    None,
}

struct StringPatternBuilder {
    bytes: Vec<u8>,
    alternatives: Vec<AlternativeRepresentation>,
    len_limit: usize,
}

impl StringPattern {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }

    pub fn unquoted(&self) -> &[u8] {
        &self.bytes[1..self.bytes.len() - 1]
    }

    pub fn quoted(&self) -> &[u8] {
        &self.bytes
    }

    pub fn len_limit(&self) -> usize {
        self.len_limit
    }

    #[inline]
    #[must_use]
    pub fn new(string: &JsonString) -> Self {
        // A pattern to be matched consists of the bytes that should be matched in the "canonical"
        // representation of the string (the shortest possible valid representation), and possible
        // alternative escapes that should be considered if a mismatch occurs
        // at a given position relative to the canonical bytes.
        // We have the following cases:
        //   - The character is a control character or a special symbol that is canonically represented
        //     as backslash-itself. If it is mismatched at the backslash, there is no match alternative
        //     representation; on the second byte it can be replaced with uXXXX.
        //   - The character is a control character that can only be represented as a unicode escape;
        //     it has no alternative encodings.
        //   - The character is one of the two awfully designed JSON special cases:
        //     forward slash (/) or single quote ('). The canonical form of them is themselves, but they
        //     can also be present escaped (\/ or \'), or as a unicode escape.
        //   - The character is a "regular" character; it has only one alternative encoding - unicode
        //     escape, which is either a single sequence \uXXXX or a pair \uXXXX\uXXXX.
        let byte_length = string.quoted().len();
        let mut builder = StringPatternBuilder::new(byte_length);

        for char in string.unquoted().chars() {
            match char {
                '\u{0008}' => builder.short_escape(b'b', char),
                '\u{000C}' => builder.short_escape(b'f', char),
                '\n' => builder.short_escape(b'n', char),
                '\r' => builder.short_escape(b'r', char),
                '\t' => builder.short_escape(b't', char),
                '"' => builder.short_escape(b'"', char),
                '\\' => builder.short_escape(b'\\', char),
                '\u{0000}'..='\u{001F}' => builder.long_escape(char),
                '/' | '\'' => builder.special_escape(char),
                _ => builder.regular_escape(char),
            };
        }

        builder.into_pattern()
    }
}

impl StringPatternBuilder {
    fn new(byte_len: usize) -> Self {
        let mut this = Self {
            bytes: Vec::with_capacity(byte_len),
            alternatives: Vec::with_capacity(byte_len),
            len_limit: 0,
        };
        this.bytes.push(b'"');
        this.alternatives.push(AlternativeRepresentation::None);
        this.len_limit += 1;

        this
    }

    fn into_pattern(mut self) -> StringPattern {
        self.bytes.push(b'"');
        self.alternatives.push(AlternativeRepresentation::None);
        self.len_limit += 1;
        let len = self.bytes.len();
        for _ in 0..BLOCK_SIZE {
            self.bytes.push(JSON_SPACE_BYTE);
        }

        return StringPattern {
            bytes: self.bytes,
            alternatives: self.alternatives,
            len_limit: self.len_limit,
            len,
        };
    }

    fn short_escape(&mut self, code_letter: u8, c: char) {
        self.bytes.push(b'\\');
        self.bytes.push(code_letter);

        let mut utf16_buf = [0; 1];
        let utf16 = c.encode_utf16(&mut utf16_buf);
        let code = Self::encode(utf16[0]);

        self.alternatives.push(AlternativeRepresentation::None);
        self.alternatives.push(AlternativeRepresentation::USingle(code));

        self.len_limit += 6;
    }

    fn long_escape(&mut self, c: char) {
        self.bytes.push(b'\\');
        self.bytes.push(b'u');
        self.bytes.push(b'0');
        self.bytes.push(b'0');
        self.bytes.push(Self::encode_nibble((c as u8 & 0xF0) >> 4));
        self.bytes.push(Self::encode_nibble(c as u8 & 0x0F));

        for _ in 0..6 {
            self.alternatives.push(AlternativeRepresentation::None);
        }

        self.len_limit += 6;
    }

    fn special_escape(&mut self, c: char) {
        self.bytes.push(c as u8);

        let mut utf16_buf = [0; 1];
        let utf16 = c.encode_utf16(&mut utf16_buf);
        let code = Self::encode(utf16[0]);

        self.alternatives
            .push(AlternativeRepresentation::SlashByteOrUSingle(c as u8, code));

        self.len_limit += 6;
    }

    fn regular_escape(&mut self, c: char) {
        let mut utf8_buf = [0; 4];
        let mut utf16_buf = [0; 2];
        let utf8 = c.encode_utf8(&mut utf8_buf);
        let utf16 = c.encode_utf16(&mut utf16_buf);

        self.bytes.extend_from_slice(utf8.as_bytes());
        let len = utf8.len();
        let repr;

        if utf16.len() == 1 {
            let code = Self::encode(utf16[0]);
            repr = AlternativeRepresentation::SlashUSingle(code, len as u8);
            self.alternatives.push(repr);
            self.len_limit += 6;
        } else {
            let code1 = Self::encode(utf16[0]);
            let code2 = Self::encode(utf16[1]);
            repr = AlternativeRepresentation::SlashUPair(code1, code2, len as u8);
            self.alternatives.push(repr);
            self.len_limit += 12;
        }

        for _ in 1..utf8.len() {
            self.alternatives.push(AlternativeRepresentation::None);
        }
        let last_idx = self.alternatives.len() - 1;
        self.alternatives[last_idx] = repr;
    }

    fn encode(utf16: u16) -> u32 {
        let bytes = utf16.to_be_bytes();
        let mut result = [0; 4];
        result[0] = Self::encode_nibble((bytes[0] & 0xF0) >> 4);
        result[1] = Self::encode_nibble(bytes[0] & 0x0F);
        result[2] = Self::encode_nibble((bytes[1] & 0xF0) >> 4);
        result[3] = Self::encode_nibble(bytes[1] & 0x0F);

        u32::from_ne_bytes(result)
    }

    fn encode_nibble(nibble: u8) -> u8 {
        match nibble {
            0x00..=0x09 => b'0' + nibble,
            0x0A..=0x0F => b'a' + nibble - 0x0A,
            _ => unreachable!(),
        }
    }
}

impl Debug for StringPattern {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return f
            .debug_struct("StringPattern")
            .field("bytes", &self.bytes.iter().copied().map(DebugByte).collect::<Vec<_>>())
            .field("as_string", &std::str::from_utf8(&self.bytes).unwrap())
            .field("alternatives", &self.alternatives)
            .finish();
    }
}

impl Debug for AlternativeRepresentation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SlashUSingle(arg0, arg1) => f
                .debug_tuple("SlashUSingle")
                .field(&DebugCode(*arg0))
                .field(arg1)
                .finish(),
            Self::SlashUPair(arg0, arg1, arg2) => f
                .debug_tuple("SlashUPair")
                .field(&DebugCode(*arg0))
                .field(&DebugCode(*arg1))
                .field(arg2)
                .finish(),
            Self::USingle(arg0) => f.debug_tuple("USingle").field(&DebugCode(*arg0)).finish(),
            Self::SlashByteOrUSingle(arg0, arg1) => f
                .debug_tuple("SlashByteOrUSingle")
                .field(&DebugByte(*arg0))
                .field(&DebugCode(*arg1))
                .finish(),
            Self::None => write!(f, "None"),
        }
    }
}

struct DebugByte(u8);
struct DebugCode(u32);

impl Debug for DebugByte {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            0x20..=0x7F => write!(f, "b'{}'", self.0 as char),
            _ => write!(f, "0x{:0>2x}", self.0),
        }
    }
}

impl Debug for DebugCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:0>8x}", self.0)
    }
}
