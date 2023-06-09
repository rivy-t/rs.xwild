//! Emulates glob (wildcard) argument expansion on Windows. No-op on other platforms.
//!
//! Unix shells expand command-line arguments like `a*`, `file.???` and pass them expanded to applications.
//! On Windows `cmd.exe` doesn't do that, so this crate emulates the expansion there.
//! Instead of `std::env::args()` use `wild::args()`, and instead of `std::env::args_os()` use `wild::args_os()`.
//!
//! The glob syntax on Windows is limited to `*`, `?`, and `[a-z]`/`[!a-z]` ranges.
//! Glob characters in quotes (`"*"`) are not expanded.
//!
//! Parsing of quoted arguments precisely follows Windows native syntax (`CommandLineToArgvW`, specifically)
//! with all its weirdness.
//!
//! ## Usage
//!
//! Use `wild::args()` instead of  `std::env::args()`.
//!
//! Use `wild::args_os()` instead of  `std::env::args_os()`.
//!
//! If you use [clap](https://crates.rs/crates/clap), use `.get_matches_from(wild::args())` instead of `.get_matches()`.

#[cfg(any(test,windows))]
extern crate glob;

#[cfg(any(test,windows))]
mod parser;

#[cfg(any(test,windows))]
mod argsiter;

#[cfg(any(test,windows))]
mod globiter;

// Iterator types
type _StringIter = Box<Iterator<Item=String>>;
type _OsStringIter = Box<Iterator<Item=std::ffi::OsString>>;

/// Returns an iterator of glob-expanded command-line arguments. Equivalent of `std::env::args()`/`std::env::args_os`.
///
/// On non-Windows platforms it returns `std::env::args()`/`std::env::args_os()` as-is,
/// assuming expansion has already been done by the shell.
///
/// On Windows it emulates the glob expansion itself.
/// The iterator will parse arguments incrementally and access
/// the file system as it parses. This allows reading potentially huge lists of
/// filenames, but it's not an atomic snapshot (use `.collect()` if you need that).
///
/// Note that `args()` (just as `std::env::args()`) will panic if OsString glob expansions are not convertible to normal Strings (UTF-8-type).
#[cfg(not(windows))]
pub fn args() -> _StringIter {
    Box::new( std::env::args() )
}

/// Returns the program arguments (glob-expanded for Windows) as a [`String`] iterator.
///
/// Note that `args()` (just as `std::env::args()`) will panic if any argument (or respective glob expansion), as an [`OsString`], is not convertible to UTF-8 [`String`].
///
/// [`String`]: https://doc.rust-lang.org/std/string/struct.String.html
/// [`OsString`]: https://doc.rust-lang.org/std/ffi/struct.OsString.html
#[cfg(windows)]
pub fn args() -> _StringIter {
    Box::new(
        args_os().map(|s| s.into_string().unwrap())
    )
}

/// Returns the program arguments (glob-expanded for Windows) as an [`OsString`](https://doc.rust-lang.org/std/ffi/struct.OsString.html) iterator.
/// # fn args_os()
#[cfg(not(windows))]
pub fn args_os() -> _OsStringIter {
    Box::new( std::env::args_os() )
}

/// Returns the program arguments (glob-expanded for Windows) as an [`OsString`] iterator.
///
/// [`String`]: https://doc.rust-lang.org/std/string/struct.String.html
/// [`OsString`]: https://doc.rust-lang.org/std/ffi/struct.OsString.html
#[cfg(windows)]
pub fn args_os() -> _OsStringIter {
    use argsiter::Args;
    Box::new(
        Args {
            args: globs(),
            current_arg_globs: None,
        }
    )
}

/// Parses `GetCommandLineW` the same way as `CommandLineToArgvW`,
/// but escapes quoted glob metacharacters `*`, `?`, `[`, `]` using `[*]` syntax.
///
/// Windows-only, unstable.
#[cfg(windows)]
#[inline]
fn globs() -> Option<globiter::GlobArgs<'static>> {
    raw_command_line().map(|cmd| globiter::GlobArgs::new(cmd))
}

#[cfg(windows)]
extern "system" {
    fn GetCommandLineW() -> *const u16;
}

#[cfg(windows)]
fn raw_command_line() -> Option<&'static [u16]> {
    unsafe {
        let line_ptr = GetCommandLineW();
        if line_ptr.is_null() {
            return None;
        }
        let mut len = 0;
        while *line_ptr.offset(len as isize) != 0 {
            len += 1;
        }
        Some(std::slice::from_raw_parts(line_ptr, len))
    }
}

#[cfg(test)]
fn parsed(s: &str) -> String {
    let t: Vec<_> = s.encode_utf16().collect();
    let args: Vec<_> = globiter::GlobArgs::new(&t)
        .map(|s| s.pattern.to_string_lossy().to_string())
        .collect();
    args.join(";")
}

#[cfg(test)]
fn unquoted(s: &str) -> String {
    let t: Vec<_> = s.encode_utf16().collect();
    let args: Vec<_> = globiter::GlobArgs::new(&t)
        .map(|s| s.text.to_string_lossy().to_string())
        .collect();
    args.join(";")
}

#[test]
#[cfg(windows)]
fn test_actual_args() {
    assert!(globs().expect("args found").count() >= 1);
}

#[test]
fn test_parse_1() {
    assert_eq!(r#"漢字"#, parsed("漢字"));
    assert_eq!(r#"漢字"#, parsed("\"漢字\""));
    assert_eq!(r#"漢\字"#, parsed("\"漢\\字\""));
    assert_eq!(r#"unquoted"#, parsed("unquoted"));
    assert_eq!(r#"*"#, parsed("*"));
    assert_eq!(r#"?"#, parsed("?"));
    assert_eq!(r#"quoted"#, parsed("\"quoted\""));
    assert_eq!(r#"quoted"#, unquoted("\"quoted\""));
    assert_eq!(r#"[*]"#, parsed("\"*\""));
    assert_eq!(r#"*"#, unquoted("\"*\""));
    assert_eq!(r#"[?]"#, parsed("\"?\""));
    assert_eq!(r#"?"#, unquoted("\"?\""));
    assert_eq!(r#"[]]"#, parsed("\"]\""));
    assert_eq!(r#"]"#, unquoted("\"]\""));
    assert_eq!(r#"quo"ted"#, parsed(r#"  "quo\"ted"  "#)); // backslash can escape quotes
    assert_eq!(r#"quo"ted?  "#, parsed(r#"  "quo""ted?"  "#)); // and quote can escape quotes
    assert_eq!(r#"unquo"ted"#, parsed(r#"  unquo\"ted  "#)); // backslash can escape quotes, even outside quotes
    assert_eq!(r#"unquoted?"#, parsed(r#"  unquo""ted?  "#)); // quote escaping does not work outside quotes
    assert_eq!(r#"""#, parsed(r#""""""#)); // quote escapes quote in quoted string
    assert_eq!(r#"""#, parsed(r#"""""""#));
    assert_eq!(r#""""#, parsed(r#""""""""#));
    assert_eq!(r#""""#, parsed(r#"""""""""#)); // """ == "X", """""" = "X""X"
    assert_eq!(r#""""#, parsed(r#""""""""""#));
    assert_eq!(r#"""""#, parsed(r#"""""""""""#));
    assert_eq!(r#"\\server\share\path with spaces"#, parsed(r#""\\server\share\path with spaces""#)); // lone double backslash is not special
    assert_eq!("aba", parsed(r#""a"b"a""#)); // quotes can go in and out
    assert_eq!("abac", parsed(r#""a"b"a"c"#)); // quotes can go in and out
    assert_eq!("c*a[*]b*a[*]c*", parsed(r#"c*"a*"b*"a*"c*"#)); // quotes can go in and out
    assert_eq!(r#"\\"#, parsed(r#"\\\\""#));
    assert_eq!(r#"?\\?"#, parsed(r#"?\\\\"?"#)); // unpaired quote is interpreted like an end quote
    assert_eq!(r#"\""#, parsed(r#"\\\""#));
    assert_eq!(r#"\"[a-z]"#, parsed(r#"\\\"[a-z]"#));
    assert_eq!("    ", parsed(r#""    "#)); // unterminated quotes are OK
    assert_eq!("", parsed(r#""""#));
    assert_eq!(r#"[a-c][d-z]"#, parsed(r#"[a-c]""[d-z]"#));
    assert_eq!(r#"[[]a-c[]]"[d-z]"#, parsed(r#""[a-c]""[d-z]""#));
    assert_eq!("", parsed(r#"""#));
    assert_eq!("x", parsed(r#"x""#));
    assert_eq!(r#"\"#, parsed(r"\"));
    assert_eq!(r#"\\"#, parsed(r"\\"));
    assert_eq!(r#"\\\"#, parsed(r"\\\"));
    assert_eq!(r#"\\\\"#, parsed(r"\\\\"));
    assert_eq!(r#"\\a"#, parsed(r#"\\\\"a"#));
    assert_eq!(r#"\\a"#, parsed(r#"\\\\"a""#));
    assert_eq!(r#"¥¥"#, parsed(r#"¥¥""#)); // in Unicode this isn't backslash
}

#[test]
fn test_parse_multi() {
    assert_eq!(r#"unquoted;quoted"#, parsed("unquoted \"quoted\""));
    assert_eq!(r#"quo"ted;quo"ted    "#, parsed(r#"  "quo\"ted"  "quo""ted"    "#));
    assert_eq!(r#"unquo"ted;""#, parsed(r#" unquo\"ted """"""#));
    assert_eq!(r#"a;a"#, parsed(r#"a"" a"#));
    assert_eq!(r#"a";a"#, parsed(r#"a""" a"#));
    assert_eq!(r#"\\;\""#, parsed(r#"\\\\"       \\\"  "#));
    assert_eq!("x;    ", parsed(r#" x  "    "#));
}
