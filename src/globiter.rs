use std::ffi::OsString;
use parser;

/// Iterator retuning glob-escaped arguments. Call `args()` to obtain it.
#[must_use]
#[derive(Debug)]
pub(crate) struct GlobArgs<'a> {
    line: &'a [u16],
}

#[cfg(windows)]
use std::os::windows::ffi::OsStringExt;

/// This is used only in tests on non-Windows
#[cfg(not(windows))]
trait LossyOsStringExt {
    fn from_wide(wide: &[u16]) -> OsString {
        OsString::from(String::from_utf16_lossy(wide))
    }
}

#[cfg(not(windows))]
impl LossyOsStringExt for OsString {}

impl<'a> Iterator for GlobArgs<'a> {
    type Item = OsString;
    fn next(&mut self) -> Option<Self::Item> {
        let (arg, rest) = parser::next_arg(self.line, vec![], |arg, c, quoted| match c as u8 {
            b'?' | b'*' | b'[' | b']' if quoted && c < 256 => {
                arg.push(u16::from(b'['));
                arg.push(c);
                arg.push(u16::from(b']'));
            },
            _ => arg.push(c),
        });
        self.line = rest;
        arg.map(|arg| OsString::from_wide(&arg))
    }
}

impl<'a> GlobArgs<'a> {
    /// UTF-16/UCS2 string from `GetCommandLineW`
    #[allow(dead_code)]
    pub(crate) fn new(line: &'a [u16]) -> Self {
        Self { line }
    }
}

