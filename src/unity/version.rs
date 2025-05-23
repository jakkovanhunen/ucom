use serde::{Deserialize, Deserializer};
use serde::{Serialize, Serializer, de};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use strum::Display;

//
// Error  implementation
//

/// Errors that can occur when parsing a [`Version`].
#[derive(Debug, Eq, PartialEq)]
pub struct ParseError;

impl std::error::Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "failed to parse Version")
    }
}

//
// Type aliases for the version components.
//

pub type Major = u16;
pub type Minor = u8;
pub type Patch = u8;
pub type BuildNumber = u8;

//
// Version components
//

/// The type of build.
#[derive(Display, Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Hash)]
pub enum BuildType {
    Alpha,
    Beta,
    ReleaseCandidate,
    Final,
    FinalPatch,
}

impl FromStr for BuildType {
    type Err = ParseError;

    /// Returns the [`BuildType`] from a string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains('f') {
            Ok(Self::Final)
        } else if s.contains('b') {
            Ok(Self::Beta)
        } else if s.contains('a') {
            Ok(Self::Alpha)
        } else if s.contains("rc") {
            Ok(Self::ReleaseCandidate)
        } else if s.contains('p') {
            Ok(Self::FinalPatch)
        } else {
            Err(ParseError)
        }
    }
}

impl BuildType {
    /// Returns the short name of the [`BuildType`].
    pub const fn to_short_str(self) -> &'static str {
        match self {
            Self::Alpha => "a",
            Self::Beta => "b",
            Self::ReleaseCandidate => "rc",
            Self::Final => "f",
            Self::FinalPatch => "p",
        }
    }
}

/// The Unity version separated into its components.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Hash)]
pub struct Version {
    pub major: Major,
    pub minor: Minor,
    pub patch: Patch,
    pub build_type: BuildType,
    pub build: BuildNumber,
}

impl FromStr for Version {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // version format: major.minor.patch.build_type.build
        let mut parts = s.split('.');

        let major = parts
            .next()
            .and_then(|s| s.parse().ok())
            .ok_or(ParseError)?;

        let minor = parts
            .next()
            .and_then(|s| s.parse().ok())
            .ok_or(ParseError)?;

        let build_part = parts.next().ok_or(ParseError)?;
        let build_type: BuildType = build_part.parse()?;

        let (patch, build) = build_part
            .split_once(build_type.to_short_str())
            .and_then(|(l, r)| l.parse().ok().zip(r.parse().ok()))
            .ok_or(ParseError)?;

        Ok(Self {
            major,
            minor,
            patch,
            build_type,
            build,
        })
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_interned_str())
    }
}

impl<'de> Deserialize<'de> for Version {
    /// Deserializes a [`Version`] from a string.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(de::Error::custom)
    }
}

impl Serialize for Version {
    /// Serializes a [`Version`] to a string.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.format_version())
    }
}

impl Version {
    /// Returns the `major.minor` part of this version.
    pub fn to_major_minor_string(self) -> String {
        format!("{}.{}", self.major, self.minor)
    }

    /// Returns the version as a static string.
    pub fn to_interned_str(self) -> &'static str {
        // Use a thread-local cache as usage is predominantly single-threaded. This avoids having to use a Mutex.
        thread_local! {
            static VERSION_STRINGS: RefCell<HashMap<Version, &'static str>> = RefCell::new(HashMap::with_capacity(100));
        }

        VERSION_STRINGS.with(|versions| {
            // Avoid borrow_mut if we already have a cached version.
            let borrow = versions.borrow();
            if let Some(&cached) = borrow.get(&self) {
                return cached;
            }
            drop(borrow);

            // If we don't have a cached version, insert it.
            let mut borrow = versions.borrow_mut();
            *borrow
                .entry(self)
                .or_insert_with(|| Box::leak(self.format_version().into_boxed_str()))
        })
    }

    /// Returns the length of the string representation of this version.
    const fn format_len(self) -> usize {
        Self::count_digits(self.major as usize)
            + Self::count_digits(self.minor as usize)
            + Self::count_digits(self.patch as usize)
            + self.build_type.to_short_str().len()
            + Self::count_digits(self.build as usize)
            + 2 // The 2 dots
    }

    const fn count_digits(number: usize) -> usize {
        match number {
            0..=9 => 1,
            10..=99 => 2,
            100..=999 => 3,
            1_000..=9_999 => 4,
            10_000..=99_999 => 5,
            100_000..=999_999 => 6,
            _ => {
                // Fall back to a loop for larger numbers (this is rare in Unity versions)
                let mut count = 0;
                let mut n = number;
                while n > 0 {
                    count += 1;
                    n /= 10;
                }
                count
            }
        }
    }

    /// Formats this version into a string.
    fn format_version(self) -> String {
        let capacity = self.format_len();
        let mut s = String::with_capacity(capacity);
        // major.minor.patch.build_type.build
        s.push_str(&self.major.to_string());
        s.push('.');
        s.push_str(&self.minor.to_string());
        s.push('.');
        s.push_str(&self.patch.to_string());
        s.push_str(self.build_type.to_short_str());
        s.push_str(&self.build.to_string());
        s
    }
}

//
// Tests
//

#[cfg(test)]
mod version_tests {
    use crate::unity::ParseError;

    use super::{BuildType, Version};

    #[test]
    fn test_version_from_string_f() {
        let v = "2021.2.14f1".parse::<Version>().unwrap();
        assert_eq!(v.major, 2021);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 14);
        assert_eq!(v.build_type, BuildType::Final);
        assert_eq!(v.build, 1);
    }

    #[test]
    fn test_version_from_string_b() {
        let v = "2021.1.1b3".parse::<Version>().unwrap();
        assert_eq!(v.major, 2021);
        assert_eq!(v.minor, 1);
        assert_eq!(v.patch, 1);
        assert_eq!(v.build_type, BuildType::Beta);
        assert_eq!(v.build, 3);
    }

    #[test]
    fn test_version_from_string_a() {
        let v = "2021.1.1a3".parse::<Version>().unwrap();
        assert_eq!(v.major, 2021);
        assert_eq!(v.minor, 1);
        assert_eq!(v.patch, 1);
        assert_eq!(v.build_type, BuildType::Alpha);
        assert_eq!(v.build, 3);
    }

    #[test]
    fn test_version_from_string_rc() {
        let v = "2021.1.1rc1".parse::<Version>().unwrap();
        assert_eq!(v.major, 2021);
        assert_eq!(v.minor, 1);
        assert_eq!(v.patch, 1);
        assert_eq!(v.build_type, BuildType::ReleaseCandidate);
        assert_eq!(v.build, 1);
    }

    #[test]
    fn test_version_from_string_invalid_build_type() {
        assert_eq!("2021.1.1x1".parse::<Version>(), Err(ParseError));
    }

    #[test]
    fn test_version_from_string_invalid() {
        assert_eq!("2021.1.1".parse::<Version>(), Err(ParseError));
    }

    #[test]
    fn test_version_to_string() {
        assert_eq!(
            "2021.2.14f1".parse::<Version>().unwrap().to_string(),
            "2021.2.14f1"
        );

        assert_eq!(
            "2019.1.1b1".parse::<Version>().unwrap().to_string(),
            "2019.1.1b1"
        );

        assert_eq!(
            "2020.1.1a3".parse::<Version>().unwrap().to_string(),
            "2020.1.1a3"
        );

        assert_eq!(
            "2022.2.1rc2".parse::<Version>().unwrap().to_string(),
            "2022.2.1rc2"
        );
    }

    #[test]
    fn test_len() {
        assert_eq!(
            "5.102.5f123".parse::<Version>().unwrap().format_len(),
            "5.102.5f123".len()
        );

        assert_eq!(
            "2021.2.14f1".parse::<Version>().unwrap().format_len(),
            "2021.2.14f1".len()
        );

        assert_eq!(
            "2019.1.1b1".parse::<Version>().unwrap().format_len(),
            "2019.1.1b1".len()
        );

        assert_eq!(
            "2020.1.1a3".parse::<Version>().unwrap().format_len(),
            "2020.1.1a3".len()
        );

        assert_eq!(
            "2022.2.1rc2".parse::<Version>().unwrap().format_len(),
            "2022.2.1rc2".len()
        );
    }
}
