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
                    Some(arg) => match glob::glob(&arg.pattern.to_string_lossy()) {
                        Ok(mut glob_iter) => {
                            let first_glob = first_non_error(&mut glob_iter);
                            self.current_arg_globs = Some(glob_iter);
                            match first_glob {
                                Some(path) => Some(path.into_os_string()),
                                None => {
                                    // non-matching patterns are passed as regular strings
                                    self.current_arg_globs = None;
                                    Some(arg.text)
                                },
                            }
                        },
                        Err(_) => {
                            // Invalid patterns are passed as regular strings
                            Some(arg.text)
                        },
                    },
                    None => None, // end of args
                },
                None => None, // error: no args available at all
            },
        }
    }
}

#[test]
fn finds_cargo_toml() {
    let cmd = "foo.exe _not_?a?_[f]ilename_ \"_not_?a?_[p]attern_\" Cargo.tom?".chars().map(|c| c as u16).collect::<Vec<_>>();
    let args = GlobArgs::new(unsafe {::std::mem::transmute(&cmd[..])});
    let iter = Args {
        args: Some(args),
        current_arg_globs: None,
    };
    let args: Vec<_> = iter.map(|c| c.to_string_lossy().to_string()).collect();
    assert_eq!(4, args.len());
    assert_eq!("foo.exe", &args[0]);
    assert_eq!("_not_?a?_[f]ilename_", &args[1]);
    assert_eq!("_not_?a?_[p]attern_", &args[2]);
    assert_eq!("Cargo.toml", &args[3]);
}
