# From OsStr

> **A macro to convert a &OsStr to a more useful types**

[![Crates.io](https://img.shields.io/crates/v/from-os-str?style=flat-square)](https://crates.io/crates/from-os-str)
[![Crates.io](https://img.shields.io/crates/d/from-os-str?style=flat-square)](https://crates.io/crates/from-os-str)
![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)
![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)
[![Build Status](https://img.shields.io/github/workflow/status/aj-bagwell/from-os-str/CI/master?style=flat-square)](https://github.com/aj-bagwell/from-os-str/actions/workflows/ci.yml?query=branch%3Amaster)

Dual-licensed under Apache 2.0 or MIT

## About

A macro for trying to convert an &OsStr to another more usefull type
There are lots of ways to do that and this will pick the best via
[autoref based specialization](https://lukaskalbertodt.github.io/2019/12/05/generalized-autoref-based-specialization.html)

e.g. a `PathBuf` will be created via `From<OsString>` not `From<String>` so non UTF8 paths
will work.

It is most useful in other macros where you don't know the type you are converting to.

## Example

```rust
use from_os_str::*;
use std::ffi::OsStr;
use std::path::Path;
let os_str = OsStr::new("123");
let path = try_from_os_str!(os_str as &Path);
assert_eq!(path, Ok(Path::new("123")));
let int = try_from_os_str!(os_str as u8);
assert_eq!(int, Ok(123));
```

## Conversion Methods

It will use one of the following traits (in order of preferece) to convert the `&OsStr` to the type you want.

* From<&OsStr>
* From<&Path>
* From<OsString>
* From<&str>
* From<String>
* TryFrom<&OsStr>
* TryFrom<&str>
* FromStr
