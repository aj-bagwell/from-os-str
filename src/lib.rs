use std::{
    convert::Infallible,
    ffi::{OsStr, OsString},
    marker::PhantomData,
    path::Path,
    str::FromStr,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error<T> {
    Utf8,
    ParseErr(T),
}

pub struct Wrap<T>(pub T);

// Generate a trait for one layer of autoref based specialization
// https://lukaskalbertodt.github.io/2019/12/05/generalized-autoref-based-specialization.html
macro_rules! specialize {
    (impl ($($and:tt)+) $name:ident for $from_ty:path {
        fn from_str($s:ident: &str) -> Result<T, $err:ty> {$($body:tt)*}
    }) => {
        specialize! {
            impl ($($and)+) $name for $from_ty {
                fn specialized(&self) -> Result<T, Error<$err>> {
                    match self.0.0.to_str() {
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
                    let $s = self.0.0;
                    $($body)*
                }
            }
        }
    };
    (impl ($($and:tt)+) $name:ident for $from_ty:path {
        fn specialized(&$self:ident) -> Result<T, $err:ty> {$($body:tt)*}
    }) => {
        pub trait $name {
            type Return;
            fn specialized(&self) -> Self::Return;
        }

        impl<'a, T: $from_ty> $name for $($and)+Wrap<(&'a OsStr, PhantomData<T>)> {
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
    impl (&&) Specialize7 for TryFrom<&'a OsStr> {
        fn from_os_str(s: &OsStr) -> Result<T, T::Error> {
            T::try_from(s)
        }
    }
}

specialize! {
    impl (&&&) Specialize6 for TryFrom<&'a str> {
        fn from_str(s: &str) -> Result<T, T::Error> {
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

#[macro_export]
macro_rules! try_from_os_str {
    ($name:ident as $typ:ty) => {
        (&&&&&&&&Wrap(($name, PhantomData::<$typ>))).specialized()
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

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
        let os_str_2 = try_from_os_str!(os_str as &OsStr);
        assert_eq!(os_str_2, Ok(os_str));

        let path = try_from_os_str!(os_str as &Path);
        assert_eq!(path, Ok(Path::new("123")));
        let str = try_from_os_str!(os_str as &str);
        assert_eq!(str, Ok("123"));
        let string = try_from_os_str!(os_str as String);
        assert_eq!(string, Ok("123".to_string()));
        let int = try_from_os_str!(os_str as u8);
        assert_eq!(int, Ok(123));

        // test priority works
        let foo = try_from_os_str!(os_str as Foo);
        assert_eq!(foo, Ok(Foo("OS: 123".to_owned())));
    }
}
