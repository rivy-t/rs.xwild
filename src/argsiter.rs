use globiter::*;
use std::ffi::OsString;
use glob;

#[cfg_attr(test, allow(dead_code))]
pub struct Args {
    pub(crate) args: Option<GlobArgs<'static>>,
    pub(crate) current_arg_globs: Option<glob::Paths>,
}

fn first_non_error<T,E,I>(iter: &mut I) -> Option<T> where I: Iterator<Item=Result<T,E>> {
    loop {
        match iter.next() {
            Some(Ok(item)) => return Some(item),
            None => return None,
            Some(Err(_)) => {},
        }
    }
}

impl Iterator for Args {
    type Item = OsString;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current_arg_globs.as_mut().and_then(first_non_error) {
            Some(path) => Some(path.into_os_string()),
            None => match self.args {
                Some(ref mut args) => match args.next() {
                    // lossy: https://github.com/rust-lang-nursery/glob/issues/23
                    Some(pattern) => match glob::glob(&pattern.to_string_lossy()) {
                        Ok(mut glob_iter) => {
                            let first_glob = first_non_error(&mut glob_iter);
                            self.current_arg_globs = Some(glob_iter);
                            match first_glob {
                                Some(path) => Some(path.into_os_string()),
                                None => {
                                    // non-matching patterns are passed as regular strings
                                    self.current_arg_globs = None;
                                    Some(pattern) // FIXME: unescape it!
                                },
                            }
                        },
                        Err(_) => {
                            // Invalid patterns are passed as regular strings
                            Some(pattern) // FIXME: unescape it!
                        },
                    },
                    None => None, // end of args
                },
                None => None, // error: no args available at all
            },
        }
    }
}
