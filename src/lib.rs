use std::{convert::Infallible, ffi::OsStr, marker::PhantomData, path::Path, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Error<T> {
    Utf8,
    ParseErr(T),
}

struct Wrap<T>(T);

// Generate a trait for one layer of autoref based specialization
// https://lukaskalbertodt.github.io/2019/12/05/generalized-autoref-based-specialization.html
macro_rules! specialize {
    (impl ($($and:tt)+) $name:ident for $from_ty:path {
        fn specialized(&$self:ident) -> Result<T, $err:ty> {$($body:tt)*}
    }) => {
        trait $name {
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
    impl (&) Specialize6 for FromStr {
        fn specialized(&self) -> Result<T, Error<T::Err>> {
            match self.0 .0.to_str() {
                None => Err(Error::Utf8),
                Some(s) => T::from_str(s).map_err(Error::ParseErr),
            }
        }
    }
}

specialize! {
    impl (&&) Specialize5 for TryFrom<&'a OsStr> {
        fn specialized(&self) -> Result<T, T::Error> {
            T::try_from(self.0 .0)
        }
    }
}

specialize! {
    impl (&&&) Specialize4 for TryFrom<&'a str> {
        fn specialized(&self) -> Result<T, Error<T::Error>> {
            match self.0 .0.to_str() {
                None => Err(Error::Utf8),
                Some(s) => T::try_from(s).map_err(Error::ParseErr),
            }
        }
    }
}

specialize! {
    impl (&&&&) Specialize3 for From<&'a Path> {
        fn specialized(&self) -> Result<T, Infallible> {
            Ok(T::from(Path::new(self.0.0)))
        }
    }
}

specialize! {
    impl (&&&&&) Specialize2 for From<&'a str> {
        fn specialized(&self) -> Result<T, Error<Infallible>> {
            match self.0 .0.to_str() {
                None => Err(Error::Utf8),
                Some(s) => Ok(T::from(s)),
            }
        }
    }
}

specialize! {
    impl (&&&&&&) Specialize1 for From<&'a OsStr> {
        fn specialized(&self) -> Result<T, Infallible> {
            Ok(T::from(self.0 .0))
        }
    }
}

#[macro_export]
macro_rules! try_from_os_str {
    ($name:ident as $typ:ty) => {
        (&&&&&&&Wrap(($name, PhantomData::<$typ>))).specialized()
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
        let path = try_from_os_str!(os_str as &Path);
        assert_eq!(path, Ok(Path::new("123")));
        let str = try_from_os_str!(os_str as &str);
        assert_eq!(str, Ok("123"));
        let int = try_from_os_str!(os_str as u8);
        assert_eq!(int, Ok(123));

        // test priority works
        let foo = try_from_os_str!(os_str as Foo);
        assert_eq!(foo, Ok(Foo("OS: 123".to_owned())));
    }
}
