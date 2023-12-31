use crate::error::{Error, InvalidUnicodeError, MissingError, ParseError};

use std::{any::type_name, env::var_os, error::Error as StdError, ffi::OsString, str::FromStr};

pub fn get(key: &str) -> Raw {
    Raw {
        key: key,
        value: var_os(key),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Raw<'k> {
    key: &'k str,
    value: Option<OsString>,
}

impl<'k> Raw<'k> {
    pub fn into_inner(self) -> Option<OsString> {
        self.value
    }

    pub fn required_unchecked(self) -> Result<Valid<'k>, Error> {
        match self.value {
            Some(osstring) => match osstring.into_string() {
                Ok(string) => Ok(Valid {
                    key: self.key,
                    value: string,
                }),
                Err(osstring) => Ok(Valid {
                    key: self.key,
                    value: osstring.to_string_lossy().into_owned(),
                }),
            },
            None => Err(MissingError {
                key: self.key.to_string(),
            }
            .into()),
        }
    }

    pub fn required_checked(self) -> Result<Valid<'k>, Error> {
        match self.value {
            Some(osstring) => match osstring.into_string() {
                Ok(string) => Ok(Valid {
                    key: self.key,
                    value: string,
                }),
                Err(osstring) => Err(InvalidUnicodeError {
                    key: self.key.to_string(),
                    value: osstring.to_string_lossy().into_owned(),
                }
                .into()),
            },
            None => Err(MissingError {
                key: self.key.to_string(),
            }
            .into()),
        }
    }

    pub fn optional_unchecked(self) -> Option<Valid<'k>> {
        match self.value {
            Some(osstring) => match osstring.into_string() {
                Ok(string) => Some(Valid {
                    key: self.key,
                    value: string,
                }),
                Err(osstring) => Some(Valid {
                    key: self.key,
                    value: osstring.to_string_lossy().into_owned(),
                }),
            },
            None => None,
        }
    }

    pub fn optional_checked(self) -> Result<Option<Valid<'k>>, Error> {
        match self.value {
            Some(osstring) => match osstring.into_string() {
                Ok(string) => Ok(Some(Valid {
                    key: self.key,
                    value: string,
                })),
                Err(osstring) => Err(InvalidUnicodeError {
                    key: self.key.to_string(),
                    value: osstring.to_string_lossy().into_owned(),
                }
                .into()),
            },
            None => Ok(None),
        }
    }

    pub fn with_default_unchecked(self, default: impl Into<String>) -> Valid<'k> {
        match self.value {
            Some(osstring) => match osstring.into_string() {
                Ok(string) => Valid {
                    key: self.key,
                    value: string,
                },
                Err(osstring) => Valid {
                    key: self.key,
                    value: osstring.to_string_lossy().into_owned(),
                },
            },
            None => Valid {
                key: self.key,
                value: default.into(),
            },
        }
    }

    pub fn with_default_unchecked_sub_invalid(self, default: impl Into<String>) -> Valid<'k> {
        match self.value {
            Some(osstring) => match osstring.into_string() {
                Ok(string) => Valid {
                    key: self.key,
                    value: string,
                },
                Err(_) => Valid {
                    key: self.key,
                    value: default.into(),
                },
            },
            None => Valid {
                key: self.key,
                value: default.into(),
            },
        }
    }

    pub fn with_default_checked(self, default: impl Into<String>) -> Result<Valid<'k>, Error> {
        match self.value {
            Some(osstring) => match osstring.into_string() {
                Ok(string) => Ok(Valid {
                    key: self.key,
                    value: string,
                }),
                Err(osstring) => Err(InvalidUnicodeError {
                    key: self.key.to_string(),
                    value: osstring.to_string_lossy().into_owned(),
                }
                .into()),
            },
            None => Ok(Valid {
                key: self.key,
                value: default.into(),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Valid<'k> {
    key: &'k str,
    value: String,
}

impl<'k> Valid<'k> {
    pub fn into_inner(self) -> String {
        self.value
    }

    pub fn then_try_fromstr_into<T>(self) -> Result<Parsed<'k, T>, Error>
    where
        T: FromStr,
        <T as FromStr>::Err: StdError + Send + Sync + 'static,
    {
        match self.value.parse() {
            Ok(parsed) => Ok(Parsed {
                inner: parsed,
                key: self.key,
                value: self.value,
            }),
            Err(err) => Err(ParseError {
                key: self.key.to_string(),
                value: self.value,
                from: type_name::<&str>(),
                to: type_name::<T>(),
                err: err.into(),
            }
            .into()),
        }
    }

    pub fn then_string_into<T>(self) -> Parsed<'k, T>
    where
        String: Into<T>,
    {
        Parsed {
            inner: self.value.clone().into(),
            key: self.key,
            value: self.value,
        }
    }

    pub fn then_try_string_into<T>(self) -> Result<Parsed<'k, T>, Error>
    where
        String: TryInto<T>,
        <String as TryInto<T>>::Error: StdError + Send + Sync + 'static,
    {
        match self.value.clone().try_into() {
            Ok(parsed) => Ok(Parsed {
                inner: parsed,
                key: self.key,
                value: self.value,
            }),
            Err(err) => Err(ParseError {
                key: self.key.to_string(),
                value: self.value,
                from: type_name::<String>(),
                to: type_name::<T>(),
                err: err.into(),
            }
            .into()),
        }
    }

    pub fn then_str_into<T>(self) -> Parsed<'k, T>
    where
        for<'v> &'v str: Into<T>,
    {
        Parsed {
            inner: self.value.as_str().into(),
            key: self.key,
            value: self.value,
        }
    }

    pub fn then_try_str_into<T>(self) -> Result<Parsed<'k, T>, Error>
    where
        for<'v> &'v str: TryInto<T>,
        for<'v> <&'v str as TryInto<T>>::Error: StdError + Send + Sync + 'static,
    {
        let parsed = self.value.as_str().try_into().map_err(|e| e.into());
        match parsed {
            Ok(parsed) => Ok(Parsed {
                inner: parsed,
                key: self.key,
                value: self.value,
            }),
            Err(err) => Err(ParseError {
                key: self.key.to_string(),
                value: self.value,
                from: type_name::<&str>(),
                to: type_name::<T>(),
                err: err,
            }
            .into()),
        }
    }

    pub fn then_fn_string_into<T, F>(self, f: F) -> Parsed<'k, T>
    where
        F: FnOnce(String) -> T,
    {
        Parsed {
            inner: f(self.value.clone()),
            key: self.key,
            value: self.value,
        }
    }

    pub fn then_try_fn_string_into<T, F, E>(self, f: F) -> Result<Parsed<'k, T>, Error>
    where
        F: FnOnce(String) -> Result<T, E>,
        E: StdError + Send + Sync + 'static,
    {
        match f(self.value.clone()) {
            Ok(parsed) => Ok(Parsed {
                inner: parsed,
                key: self.key,
                value: self.value,
            }),
            Err(err) => Err(ParseError {
                key: self.key.to_string(),
                value: self.value,
                from: type_name::<String>(),
                to: type_name::<T>(),
                err: err.into(),
            }
            .into()),
        }
    }

    pub fn then_fn_str_into<T, F>(self, f: F) -> Parsed<'k, T>
    where
        F: for<'v> FnOnce(&'v str) -> T,
    {
        Parsed {
            inner: f(self.value.as_str()),
            key: self.key,
            value: self.value,
        }
    }

    pub fn then_try_fn_str_into<T, F, E>(self, f: F) -> Result<Parsed<'k, T>, Error>
    where
        F: for<'v> FnOnce(&'v str) -> Result<T, E>,
        E: StdError + Send + Sync + 'static,
    {
        match f(self.value.as_str()) {
            Ok(parsed) => Ok(Parsed {
                inner: parsed,
                key: self.key,
                value: self.value,
            }),
            Err(err) => Err(ParseError {
                key: self.key.to_string(),
                value: self.value,
                from: type_name::<&str>(),
                to: type_name::<T>(),
                err: err.into(),
            }
            .into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parsed<'k, P> {
    key: &'k str,
    value: String,
    inner: P,
}

impl<'k, P> Parsed<'k, P> {
    pub fn into_inner(self) -> P {
        self.inner
    }

    pub fn then_into<T>(self) -> Parsed<'k, T>
    where
        P: Into<T>,
    {
        Parsed::<'k, T> {
            inner: self.inner.into(),
            key: self.key,
            value: self.value,
        }
    }

    pub fn then_try_into<T>(self) -> Result<Parsed<'k, T>, Error>
    where
        P: TryInto<T>,
        <P as TryInto<T>>::Error: StdError + Send + Sync + 'static,
    {
        match self.inner.try_into() {
            Ok(parsed) => Ok(Parsed {
                inner: parsed,
                key: self.key,
                value: self.value,
            }),
            Err(err) => Err(ParseError {
                key: self.key.to_string(),
                value: self.value,
                from: type_name::<P>(),
                to: type_name::<T>(),
                err: err.into(),
            }
            .into()),
        }
    }

    pub fn then_fn_into<T, F>(self, f: F) -> Parsed<'k, T>
    where
        F: FnOnce(P) -> T,
    {
        Parsed {
            inner: f(self.inner),
            key: self.key,
            value: self.value,
        }
    }

    pub fn then_try_fn_into<T, F, E>(self, f: F) -> Result<Parsed<'k, T>, Error>
    where
        F: FnOnce(P) -> Result<T, E>,
        E: StdError + Send + Sync + 'static,
    {
        match f(self.inner) {
            Ok(parsed) => Ok(Parsed {
                inner: parsed,
                key: self.key,
                value: self.value,
            }),
            Err(err) => Err(ParseError {
                key: self.key.to_string(),
                value: self.value,
                from: type_name::<P>(),
                to: type_name::<T>(),
                err: err.into(),
            }
            .into()),
        }
    }
}
