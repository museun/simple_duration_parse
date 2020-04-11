#[derive(Debug, PartialEq)]
pub enum Error {
    OutOfOrder,
    AlreadySeen,
    InvalidData,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OutOfOrder => write!(f, "Out of order"),
            Self::AlreadySeen => write!(f, "Already seen"),
            Self::InvalidData => write!(f, "Invalid data"),
        }
    }
}

impl std::error::Error for Error {}

/// Parse the input string into a type
///
/// ```rust
/// use simple_duration_parse::DurationParser as _;
/// use std::time::Duration;
///
/// assert_eq!(Duration::parse_human_duration("7d 3m").unwrap(), Duration::from_secs(604980));
/// ```
pub trait DurationParser {
    fn parse_human_duration(input: &str) -> Result<Self, Error>
    where
        Self: Sized;
}

impl DurationParser for std::time::Duration {
    fn parse_human_duration(input: &str) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let secs = parse_secs(input)?;
        Ok(std::time::Duration::from_secs(secs))
    }
}

#[cfg(feature = "time")]
impl DurationParser for time::Duration {
    fn parse_human_duration(input: &str) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let secs = parse_secs(input)?;
        Ok(time::Duration::seconds(secs as _))
    }
}

/// Parse the input string into seconds
///
/// # Format:
/// | suffix | description |
/// | -- | -- |
/// | d | days |
/// | h | hours |
/// | m | minutes |
/// | s | seconds |
///
/// ```rust
/// let tests = &[
///     ("1s", 1),
///     ("1m", 60),
///     ("1h", 60 * 60),
///     ("1h 1m 1s", (60 * 60) + 60 + 1),
///     ("1h 1s", (60 * 60) + 1),
///     ("30m 59s", (30 * 60) + 59),
///     ("7d", (60 * 60 * 24 * 7)),
///     ("3d 5m", (60 * 60 * 24 * 3) + 5 * 60),
///     ("1s foobar", 1),
/// ];
///
/// for (input, expected) in tests {
///     assert_eq!(simple_duration_parse::parse_secs(&input).unwrap(), *expected);
/// }
/// ```
pub fn parse_secs(input: &str) -> Result<u64, Error> {
    #[derive(Default)]
    struct Buf(Vec<char>);
    impl Buf {
        fn is_empty(&self) -> bool {
            self.0.is_empty()
        }
        fn append(&mut self, ch: char) {
            self.0.push(ch)
        }
        fn parse(&mut self, magnitude: Magnitude) -> Option<u64> {
            if self.is_empty() {
                return None;
            }

            Some(
                self.0
                    .drain(..)
                    .filter_map(|c| c.to_digit(10).map(u64::from))
                    .fold(0, |a, c| 10 * a + c)
                    * magnitude.to_secs(),
            )
        }
    }

    #[derive(Default)]
    struct Order(Option<Magnitude>);
    impl Order {
        fn verify(&mut self, magnitude: Magnitude) -> Result<Magnitude, Error> {
            match self.0 {
                Some(a) if a > magnitude => self.0.replace(magnitude),
                Some(a) if a == magnitude => return Err(Error::AlreadySeen),
                None => self.0.replace(magnitude),
                _ => return Err(Error::OutOfOrder),
            };
            Ok(magnitude)
        }
    }

    #[derive(Copy, Clone, PartialEq, PartialOrd)]
    enum Magnitude {
        Second,
        Minute,
        Hour,
        Day,
    }
    impl Magnitude {
        fn to_secs(self) -> u64 {
            match self {
                Self::Second => 1,
                Self::Minute => 60,
                Self::Hour => 60 * 60,
                Self::Day => 60 * 60 * 24,
            }
        }
    }

    let (mut order, mut buf): (Order, Buf) = Default::default();
    let mut iter = input.chars().peekable();
    let mut acc = 0;

    macro_rules! verify {
        ($mag:expr) => {{
            if buf.is_empty() {
                return Err(Error::InvalidData);
            }
            match buf.parse(order.verify($mag)?) {
                Some(d) => d,
                None => break,
            }
        }};
    }

    while let Some(left) = iter.next() {
        acc += match (left, iter.peek()) {
            ('s', ..) => verify!(Magnitude::Second),
            ('m', ..) => verify!(Magnitude::Minute),
            ('h', ..) => verify!(Magnitude::Hour),
            ('d', ..) => verify!(Magnitude::Day),
            (c, Some(..)) if c.is_ascii_digit() => {
                if buf.is_empty() && c == '0' {
                    return Err(Error::InvalidData);
                }
                buf.append(c);
                continue;
            }
            (c, None) if c.is_ascii_digit() => return Err(Error::InvalidData),
            _ => continue,
        }
    }

    return Ok(acc);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_secs_test() {
        let tests = &[
            ("1s", 1),
            ("1m", 60),
            ("1h", 60 * 60),
            ("1h 1m 1s", (60 * 60) + 60 + 1),
            ("1h 1s", (60 * 60) + 1),
            ("30m 59s", (30 * 60) + 59),
            ("7d", (60 * 60 * 24 * 7)),
            ("3d 5m", (60 * 60 * 24 * 3) + 5 * 60),
            ("1s foobar", 1),
        ];

        for (input, expected) in tests {
            assert_eq!(parse_secs(&input).unwrap(), *expected, "input: {}", input);
        }

        let tests = &[
            ("1s 1m", Error::OutOfOrder),
            ("1s 1s", Error::AlreadySeen),
            ("0s", Error::InvalidData),
            ("06s", Error::InvalidData),
            ("1m 1", Error::InvalidData),
            ("1s1", Error::InvalidData),
        ];

        for (input, expected) in tests {
            assert_eq!(
                parse_secs(&input).unwrap_err(),
                *expected,
                "input: {}",
                input
            );
        }
    }
}

#[cfg(all(doctest))]
doc_comment::doctest!("../README.md");
