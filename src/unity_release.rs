use std::borrow::Cow;

use anyhow::Result;
use indexmap::IndexMap;
use select::document::Document;
use select::predicate::{Class, Name};

use crate::unity_version::*;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ReleaseInfo {
    pub version: UnityVersion,
    pub date_header: String,
    pub installation_url: String,
}

impl ReleaseInfo {
    const fn new(version: UnityVersion, date_header: String, installation_url: String) -> Self {
        Self {
            version,
            date_header,
            installation_url,
        }
    }
}

#[allow(dead_code)]
pub enum ReleaseFilter {
    All,
    Year {
        year: VersionYear,
    },
    Point {
        year: VersionYear,
        point: VersionPoint,
    },
}

impl ReleaseFilter {
    const fn eval(&self, v: UnityVersion) -> bool {
        match self {
            Self::All => true,
            Self::Year { year } => v.year == *year,
            Self::Point { year, point } => v.year == *year && v.point == *point,
        }
    }
}

/// Gets Unity releases from the Unity website.
pub fn request_unity_releases() -> Result<Vec<ReleaseInfo>> {
    let url = "https://unity.com/releases/editor/archive";
    let body = ureq::get(url).call()?.into_string()?;
    let releases = find_releases(&body, &ReleaseFilter::All);
    Ok(releases)
}

/// Gets updates for the given version from the Unity website.
pub fn request_updates_for(version: UnityVersion) -> Result<Vec<ReleaseInfo>> {
    let url = "https://unity.com/releases/editor/archive";
    let body = ureq::get(url).call()?.into_string()?;

    let releases = find_releases(
        &body,
        &ReleaseFilter::Point {
            year: version.year,
            point: version.point,
        },
    )
    .into_iter()
    .filter(|ri| ri.version > version)
    .collect();

    Ok(releases)
}

/// Finds releases in the html that match the filter.
fn find_releases(html: &str, filter: &ReleaseFilter) -> Vec<ReleaseInfo> {
    let year_class: Cow<'_, str> = match filter {
        ReleaseFilter::All => "release-tab-content".into(),
        ReleaseFilter::Year { year } | ReleaseFilter::Point { year, .. } => year.to_string().into(),
    };

    let mut versions: Vec<_> = Document::from(html)
        .find(Class(year_class.as_ref()))
        .flat_map(|n| n.find(Class("download-release-wrapper")))
        .filter_map(|n| {
            n.find(Class("release-title-date"))
                .next()
                // Get the release date.
                .and_then(|n| n.find(Class("release-date")).next())
                .map(|n| n.text())
                .and_then(|date_header| {
                    // Get the Unity Hub installation url.
                    n.find(Class("btn"))
                        .next()
                        .and_then(|n| n.attr("href"))
                        .map(|url| (date_header, url))
                })
        })
        .filter_map(|(date_header, url)| {
            version_from_url(url)
                .filter(|&v| filter.eval(v))
                .map(|version| ReleaseInfo::new(version, date_header, url.to_owned()))
        })
        .collect();

    versions.sort_unstable();
    versions
}

/// Get the version from the url.
/// The url looks like: unityhub://2021.2.14f1/bcb93e5482d2
fn version_from_url(url: &str) -> Option<UnityVersion> {
    url.split('/')
        .rev()
        .nth(1)
        .and_then(|v| v.parse::<UnityVersion>().ok())
}

pub fn release_notes_url(version: UnityVersion) -> String {
    let version = if version.build == 1 {
        format!("{}.{}.{}", version.year, version.point, version.patch)
    } else {
        format!(
            "{}.{}.{}-{}",
            version.year,
            version.point,
            version.patch,
            version.build - 2
        )
    };
    format!("https://unity.com/releases/editor/whats-new/{version}")
}

pub fn collect_release_notes(html: &str) -> IndexMap<String, Vec<String>> {
    let document = Document::from(html);
    let mut release_notes = IndexMap::<String, Vec<String>>::new();

    if let Some(node) = document.find(Class("release-notes")).next() {
        let mut topic_header = "General".to_string();
        node.children().for_each(|n| match n.name() {
            Some("h3" | "h4") => topic_header = n.text(),
            Some("ul") => {
                if !release_notes.contains_key(&topic_header) {
                    release_notes.insert(topic_header.clone(), Vec::new());
                }

                let topic_list = release_notes.get_mut(&topic_header).unwrap();
                n.find(Name("li")).for_each(|li| {
                    if let Some(release_note_line) = li.text().lines().next() {
                        topic_list.push(release_note_line.to_string());
                    }
                });
            }
            _ => (),
        });
    }

    release_notes
}

#[cfg(test)]
mod releases_tests {
    use std::str::FromStr;

    use crate::unity_release::{find_releases, version_from_url, ReleaseFilter};
    use crate::unity_version::UnityVersion;

    #[test]
    fn test_version_from_url() {
        let url = "unityhub://2021.2.14f1/bcb93e5482d2";
        let version = version_from_url(url).unwrap();
        assert_eq!(version, UnityVersion::from_str("2021.2.14f1").unwrap());
    }

    #[test]
    fn test_version_from_url_invalid_url() {
        let url = "unityhub://2021.2.14f1";
        let version = version_from_url(url);
        assert!(version.is_none());
    }

    #[test]
    fn test_version_from_url_invalid_version() {
        let url = "unityhub://2021.2.14/bcb93e5482d2";
        let version = version_from_url(url);
        assert!(version.is_none());
    }

    #[test]
    fn test_find_releases_all() {
        let html = include_str!("../test_data/unity_download_archive.html");
        let releases = find_releases(html, &ReleaseFilter::All);
        assert_eq!(releases.len(), 473);
    }

    #[test]
    fn test_find_releases_year() {
        let html = include_str!("../test_data/unity_download_archive.html");
        let releases = find_releases(html, &ReleaseFilter::Year { year: 2021 });
        assert_eq!(releases.len(), 66);
    }

    #[test]
    fn test_find_releases_point() {
        let html = include_str!("../test_data/unity_download_archive.html");
        let releases = find_releases(
            html,
            &ReleaseFilter::Point {
                year: 2019,
                point: 2,
            },
        );
        assert_eq!(releases.len(), 22);
    }

    #[test]
    fn test_release_notes_url() {
        let version = UnityVersion::from_str("2021.2.14f1").unwrap();
        let url = super::release_notes_url(version);
        assert_eq!(url, "https://unity.com/releases/editor/whats-new/2021.2.14");
    }

    #[test]
    fn test_release_notes_url_5_1_0_1() {
        let version = UnityVersion::from_str("5.1.0f1").unwrap();
        let url = super::release_notes_url(version);
        assert_eq!(url, "https://unity.com/releases/editor/whats-new/5.1.0");
    }

    #[test]
    fn test_release_notes_url_5_1_0_2() {
        let version = UnityVersion::from_str("5.1.0f2").unwrap();
        let url = super::release_notes_url(version);
        assert_eq!(url, "https://unity.com/releases/editor/whats-new/5.1.0-0");
    }

    #[test]
    fn test_release_notes_url_5_1_0_3() {
        let version = UnityVersion::from_str("5.1.0f3").unwrap();
        let url = super::release_notes_url(version);
        assert_eq!(url, "https://unity.com/releases/editor/whats-new/5.1.0-1");
    }

    #[test]
    fn test_release_notes_5_0_0() {
        let html = include_str!("../test_data/unity_5_0_0.html");
        let release_notes = super::collect_release_notes(html);
        assert_eq!(release_notes.len(), 47);
        assert_eq!(release_notes.values().flatten().count(), 1114);
    }

    #[test]
    fn test_release_notes_2017_1_0() {
        let html = include_str!("../test_data/unity_2017_1_0.html");
        let release_notes = super::collect_release_notes(html);
        assert_eq!(release_notes.len(), 6);
        assert_eq!(release_notes.values().flatten().count(), 440);
    }

    #[test]
    fn test_release_notes_2017_2_5() {
        let html = include_str!("../test_data/unity_2017_2_5.html");
        let release_notes = super::collect_release_notes(html);
        assert_eq!(release_notes.len(), 1);
        assert_eq!(release_notes.values().flatten().count(), 10);
    }

    #[test]
    fn test_release_notes_2021_3_17() {
        let html = include_str!("../test_data/unity_2021_3_17.html");
        let release_notes = super::collect_release_notes(html);
        assert_eq!(release_notes.len(), 7);
        assert_eq!(release_notes.values().flatten().count(), 204);
    }

    #[test]
    fn test_release_notes_2022_2_0() {
        let html = include_str!("../test_data/unity_2022_2_0.html");
        let release_notes = super::collect_release_notes(html);
        assert_eq!(release_notes.len(), 7);
        assert_eq!(release_notes.values().flatten().count(), 2090);
    }
}
