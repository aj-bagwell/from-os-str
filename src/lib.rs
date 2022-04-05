#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![forbid(missing_docs)]
#![deny(warnings)]
//! A macro for trying to convert an &OsStr to another more usefull type
//! There are lots of ways to do that and this will pick the best via
//! [autoref based specialization](https://lukaskalbertodt.github.io/2019/12/05/generalized-autoref-based-specialization.html)
//! e.g. a `PathBuf` will be created via `From<OsString>` not `From<String>` so non UTF8 paths
//! will work.
//! ```
//! # #[macro_use] extern crate from_os_str;
//! # fn main() {
//! use from_os_str::*;
//! use std::ffi::OsStr;
//! use std::path::Path;
//! let os_str = OsStr::new("123");
//! let path = try_from_os_str!(os_str as &Path);
//! assert_eq!(path, Ok(Path::new("123")));
//! let int = try_from_os_str!(os_str as u8);
//! assert_eq!(int, Ok(123));
//! # }
//! ```

use std::{
    convert::Infallible,
    error::Error as StdError,
    ffi::{OsStr, OsString},
    fmt::Display,
    marker::PhantomData,
    path::Path,
    str::FromStr,
};

/// An error that can occure when converting an OsString to another type
/// It can either be a problem converting the bytes passed into the OsStr
/// as a valid UTF8 string or an error parsing the string
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error<T> {
    /// The OsStr contains bytes that are not valid UTF8
    Utf8,
    /// Parsing the string failed
    ParseErr(T),
}

impl<T: Display> Display for Error<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Utf8 => write!(f, "invalid utf-8 sequence"),
            Error::ParseErr(err) => err.fmt(f),
        }
    }
}

impl<T: StdError + 'static> StdError for Error<T> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Utf8 => None,
            Error::ParseErr(err) => Some(err),
        }
    }
}

#[doc(hidden)]
pub struct Wrap<'a, T>(&'a OsStr, PhantomData<T>);

impl<'a, T> Wrap<'a, T> {
    pub fn new(s: &'a OsStr) -> Self {
        Wrap(s, PhantomData::<T>)
    }
}

// Generate a trait for one layer of autoref based specialization
// https://lukaskalbertodt.github.io/2019/12/05/generalized-autoref-based-specialization.html
macro_rules! specialize {
    (impl ($($and:tt)+) $name:ident for $from_ty:path {
        fn from_str($s:ident: &str) -> Result<T, $err:ty> {$($body:tt)*}
    }) => {
        specialize! {
            impl ($($and)+) $name for $from_ty {
                fn specialized(&self) -> Result<T, Error<$err>> {
                    match self.0.to_str() {
                        None => Err(Error::Utf8),
                        Some($s) => {$($body)*}.map_err(Error::ParseErr),
                    }
                }
            }
        }
    };
    (impl ($($and:tt)+) $name:ident for $from_ty:path {
        fn from_os_str($s:ident: &OsStr) -> Result<T, $err:ty> {$($body:tt)*}
    }) => {
        specialize! {
            impl ($($and)+) $name for $from_ty {
                fn specialized(&self) -> Result<T, $err> {
                    let $s = self.0;
                    $($body)*
                }
            }
        }
    };
    (impl ($($and:tt)+) $name:ident for $from_ty:path {
        fn specialized(&$self:ident) -> Result<T, $err:ty> {$($body:tt)*}
    }) => {
        #[doc(hidden)]
        pub trait $name {
            type Return;
            fn specialized(&self) -> Self::Return;
        }

        impl<'a, T: $from_ty> $name for $($and)+Wrap<'a, T> {
            type Return = Result<T, $err>;
            fn specialized(&$self) -> Self::Return {$($body)*}
        }
    };
}

// Conversions from lowest priority to heighest
specialize! {
    impl (&) Specialize8 for FromStr {
        fn from_str(s: &str) -> Result<T, T::Err> {
            T::from_str(s)
        }
    }
}

specialize! {
    impl (&&) Specialize7 for TryFrom<&'a str> {
        fn from_str(s: &str) -> Result<T, T::Error> {
            T::try_from(s)
        }
    }
}

specialize! {
    impl (&&) Specialize6 for TryFrom<&'a OsStr> {
        fn from_os_str(s: &OsStr) -> Result<T, T::Error> {
            T::try_from(s)
        }
    }
}

specialize! {
    impl (&&&&) Specialize5 for From<String> {
        fn from_str(s: &str) -> Result<T, Infallible> {
            Ok(T::from(s.to_string()))
        }
    }
}

specialize! {
    impl (&&&&&) Specialize4 for From<&'a str> {
        fn from_str(s: &str) -> Result<T, Infallible> {
            Ok(T::from(s))
        }
    }
}

specialize! {
    impl (&&&&&&) Specialize3 for From<OsString> {
        fn from_os_str(s: &OsStr) -> Result<T, Infallible> {
            Ok(T::from(s.to_os_string()))
        }
    }
}

specialize! {
    impl (&&&&&&&) Specialize2 for From<&'a Path> {
        fn from_os_str(s: &OsStr) -> Result<T, Infallible> {
            Ok(T::from(Path::new(s)))
        }
    }
}

specialize! {
    impl (&&&&&&&&) Specialize1 for From<&'a OsStr> {
        fn from_os_str(s: &OsStr) -> Result<T, Infallible> {
            Ok(T::from(s))
        }
    }
}

/// Convert an `&OsStr` to another more usefull type
/// There are lots of ways to do that and this will pick the best via
/// [autoref based specialization](https://lukaskalbertodt.github.io/2019/12/05/generalized-autoref-based-specialization.html)
/// e.g. a `PathBuf` will becreated via `From<OsString>` not `From<String>` so non UTF8 paths
/// will work.
/// ```
/// # #[macro_use] extern crate from_os_str;
/// # fn main() -> Result<(), Box<dyn std::error::Error>>{
/// use from_os_str::*;
/// use std::ffi::OsStr;
/// use std::path::Path;
/// let os_str = OsStr::new("123");
/// let path = try_from_os_str!(os_str as &Path)?;
/// assert_eq!(path, Path::new("123"));
/// let str = try_from_os_str!(os_str as &str)?;
/// assert_eq!(str, "123");
/// let string = try_from_os_str!(os_str as String)?;
/// assert_eq!(string, "123".to_string());
/// let int = try_from_os_str!(os_str as u8)?;
/// assert_eq!(int, 123);
/// # Ok(())}
/// ```
#[macro_export]
macro_rules! try_from_os_str {
    ($name:ident as $typ:ty) => {
        (&&&&&&&&Wrap::<$typ>::new($name)).specialized()
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::path::PathBuf;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Foo(String);

    impl From<&OsStr> for Foo {
        fn from(s: &OsStr) -> Self {
            Foo("OS: ".to_string() + &s.to_string_lossy())
        }
    }

    impl FromStr for Foo {
        type Err = Infallible;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Foo("STR: ".to_string() + s))
        }
    }

    #[test]
    fn it_works() {
        let os_str = OsStr::new("123");
        let os_str_2 = try_from_os_str!(os_str as &OsStr).unwrap();
        assert_eq!(os_str_2, os_str);

        let path = try_from_os_str!(os_str as &Path).unwrap();
        assert_eq!(path, Path::new("123"));
        let path = try_from_os_str!(os_str as PathBuf).unwrap();
        assert_eq!(&path, Path::new("123"));

        let str = try_from_os_str!(os_str as &str).unwrap();
        assert_eq!(str, "123");
        let string = try_from_os_str!(os_str as String).unwrap();
        assert_eq!(string, "123".to_string());
        let int = try_from_os_str!(os_str as u8).unwrap();
        assert_eq!(int, 123);

        // test priority works
        let foo = try_from_os_str!(os_str as Foo);
        assert_eq!(foo, Ok(Foo("OS: 123".to_owned())));
    }

    #[test]
    #[cfg(unix)]
    fn it_works_with_non_utf8() {
        use std::os::unix::ffi::OsStrExt;
        let os_str = OsStr::from_bytes(&[0xff, 0xff]);
        let os_str_2 = try_from_os_str!(os_str as &OsStr).unwrap();
        assert_eq!(os_str_2, os_str);

        let path = try_from_os_str!(os_str as &Path).unwrap();
        assert_eq!(path, Path::new(os_str));
        let path = try_from_os_str!(os_str as PathBuf).unwrap();
        assert_eq!(path, Path::new(os_str));
        let str = try_from_os_str!(os_str as &str);
        assert_eq!(str, Err(Error::Utf8));
        let string = try_from_os_str!(os_str as String);
        assert_eq!(string, Err(Error::Utf8));
        let int = try_from_os_str!(os_str as u8);
        assert_eq!(int, Err(Error::Utf8));

        // test priority works
        let foo = try_from_os_str!(os_str as Foo);
        assert_eq!(foo, Ok(Foo("OS: ��".to_owned())));
    }
}
