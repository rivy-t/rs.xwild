# Wild::args for Rust

Emulates glob (wildcard) argument expansion on Windows. No-op on other platforms.

Unix shells expand command-line arguments like `a*`, `file.???` and pass them expanded to applications.
On Windows `cmd.exe` doesn't do that, so this crate emulates the expansion there.
Instead of `std::env::args()` use `wild::args()`.

The glob syntax on Windows is limited to `*`, `?`, and `[a-z]`/`[!a-z]` ranges.
Glob characteres in quotes (`"*"`) are not expanded.

Parsing of quoted arguments precisely follows Windows native syntax (`CommandLineToArgvW`, specifically)
with all its weirdness.
