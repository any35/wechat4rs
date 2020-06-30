#[allow(dead_code)]
pub mod iso_date_format {
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

    pub fn str2date(date: String) -> anyhow::Result<DateTime<Utc>, String> {
        let result = Utc
            .datetime_from_str(date.as_str(), FORMAT)
            .map_err(|_| format!("[{}] is NOT a valie date", date))?;
        Ok(result)
    }

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Utc.datetime_from_str(&s, FORMAT)
            .map_err(serde::de::Error::custom)
    }
}

/// #[derive(Deserialize, Debug)]
/// struct Tachyon {
///   #[serde(deserialize_with = "bool_from_int")]
///   value: bool,
/// }
pub mod int2bool_format {
    use serde::de::{Deserialize, Deserializer};

    pub fn bool_from_int<'de, D>(deserializer: D) -> Result<bool, D::Error>
    where
        D: Deserializer<'de>,
    {
        match u8::deserialize(deserializer)? {
            0 => Ok(false),
            _ => Ok(true),
        }
    }
}

pub mod str_ext {

    #[derive(Copy, Clone, Debug, PartialEq)]
    pub enum SplitType<'a> {
        Match(&'a str),
        Delimiter(&'a str),
    }

    pub struct SplitKeepingDelimiter<'a, F> {
        haystack: &'a str,
        chars: F,
        start: usize,
        saved: Option<usize>,
    }

    impl<'a, F> Iterator for SplitKeepingDelimiter<'a, F>
    where
        F: Fn(char) -> bool,
    {
        type Item = SplitType<'a>;

        fn next(&mut self) -> Option<SplitType<'a>> {
            if self.start == self.haystack.len() {
                return None;
            }

            if let Some(end_of_match) = self.saved.take() {
                let s = &self.haystack[self.start..end_of_match];
                self.start = end_of_match;
                return Some(SplitType::Delimiter(s));
            }

            let tail = &self.haystack[self.start..];

            match tail.find(&self.chars) {
                Some(start) => {
                    let start = self.start + start;
                    let end = start + 1; // Super dangerous! Assume we are only one byte long
                    if self.start == start {
                        let s = &self.haystack[start..end];
                        self.start = end;
                        Some(SplitType::Delimiter(s))
                    } else {
                        let s = &self.haystack[self.start..start];
                        self.start = start;
                        self.saved = Some(end);
                        Some(SplitType::Match(s))
                    }
                }
                None => {
                    let s = &self.haystack[self.start..];
                    self.start = self.haystack.len();
                    Some(SplitType::Match(s))
                }
            }
        }
    }

    pub trait SplitKeepingDelimiterExt:
        ::std::ops::Index<::std::ops::RangeFull, Output = str>
    {
        fn split_keeping_delimiter<F>(&self, chars: F) -> SplitKeepingDelimiter<'_, F>
        where
            F: Fn(char) -> bool,
        {
            SplitKeepingDelimiter {
                haystack: &self[..],
                chars,
                start: 0,
                saved: None,
            }
        }
    }

    impl SplitKeepingDelimiterExt for str {}

    #[cfg(test)]
    mod test {
        use super::SplitKeepingDelimiterExt;

        #[test]
        fn split_with_delimiter() {
            use super::SplitType::*;
            let delims = |b| b == ',' || b == ';';
            let items: Vec<_> = "alpha,beta;gamma".split_keeping_delimiter(delims).collect();
            assert_eq!(
                &items,
                &[
                    Match("alpha"),
                    Delimiter(","),
                    Match("beta"),
                    Delimiter(";"),
                    Match("gamma")
                ]
            );
        }

        #[test]
        fn split_with_delimiter_allows_consecutive_delimiters() {
            use super::SplitType::*;
            let delims = |b| b == ',' || b == ';';
            let items: Vec<_> = ",;".split_keeping_delimiter(delims).collect();
            assert_eq!(&items, &[Delimiter(","), Delimiter(";")]);
        }
    }
}
