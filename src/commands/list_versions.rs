use std::cmp::Ordering;
use std::env;

use anyhow::anyhow;
use colored::{ColoredString, Colorize};
use spinoff::{spinners, Spinner};

use crate::cli::{ListType, ENV_DEFAULT_VERSION};
use crate::unity::*;

/// Lists installed Unity versions.
pub fn list_versions(list_type: ListType, partial_version: Option<&str>) -> anyhow::Result<()> {
    let dir = editor_parent_dir()?;
    let matching_versions = matching_versions(available_unity_versions(&dir)?, partial_version)?;

    match list_type {
        ListType::Installed => {
            println!(
                "{}",
                format!("Unity versions in `{}`", dir.display()).bold()
            );

            print_installed_versions(&matching_versions, &Vec::new())?;
        }
        ListType::Updates => {
            println!(
                "{}",
                format!("Updates for Unity versions in `{}`", dir.display()).bold()
            );
            let spinner = Spinner::new(spinners::Dots, "Downloading release data...", None);
            let releases = request_unity_releases()?;
            spinner.clear();
            print_installed_versions(&matching_versions, &releases)?;
        }
        ListType::Latest => {
            println!("{}", "Latest available minor releases".bold());
            let spinner = Spinner::new(spinners::Dots, "Downloading release data...", None);
            let releases = request_unity_releases()?;
            spinner.clear();
            print_latest_versions(&matching_versions, &releases, partial_version);
        }
    }

    Ok(())
}

fn print_installed_versions(
    installed: &[UnityVersion],
    available: &[ReleaseInfo],
) -> anyhow::Result<()> {
    let default_version = env::var_os(ENV_DEFAULT_VERSION)
        .and_then(|env| {
            installed
                .iter()
                .rev()
                .find(|v| v.to_string().starts_with(env.to_string_lossy().as_ref()))
        })
        .or_else(|| installed.last())
        .copied()
        .ok_or_else(|| anyhow!("No Unity versions installed"))?;

    let installed: Vec<_> = installed.iter().map(|v| (v, v.to_string())).collect();
    let max_len = installed.iter().map(|(_, s)| s.len()).max().unwrap();

    let mut previous_range = None;
    let mut iter = installed.iter().peekable();

    while let Some((&version, version_string)) = iter.next() {
        let is_next_in_same_range = iter.peek().map_or(false, |(v, _)| {
            v.major == version.major && v.minor == version.minor
        });

        print_list_marker(
            Some((version.major, version.minor)) == previous_range,
            is_next_in_same_range,
        );

        previous_range = Some((version.major, version.minor));

        let mut colorize_line: fn(&str) -> ColoredString = |s: &str| ColoredString::from(s);
        let mut line = format!("{version_string:<max_len$}");

        if !available.is_empty() && !is_next_in_same_range {
            let range: Vec<_> = available
                .iter()
                .filter(|r| r.version.major == version.major && r.version.minor == version.minor)
                .collect();

            if let Some(latest) = range.last() {
                match version.cmp(&latest.version) {
                    Ordering::Equal => {
                        // Latest version in the range.
                        line.push_str(" - Up to date");
                    }
                    Ordering::Less => {
                        // Later version available.
                        colorize_line = |s: &str| s.yellow().bold();
                        line.push_str(&format!(
                            " - Update available: {} behind {} ({})",
                            range.iter().filter(|v| v.version > version).count(),
                            latest.version,
                            latest.date_header
                        ));
                    }
                    Ordering::Greater => {
                        // Installed version is newer than latest available.
                        line.push_str(" - Newer than latest available");
                    }
                }
            } else {
                // No releases in the x.y range.
                line.push_str(" - No update information available");
            }
        }

        if version == default_version {
            line.push_str(" *default for new projects");
            println!("{}", colorize_line(&line).bold());
        } else {
            println!("{}", colorize_line(&line));
        }
    }

    Ok(())
}

fn print_latest_versions(
    installed: &[UnityVersion],
    available: &[ReleaseInfo],
    partial_version: Option<&str>,
) {
    // Get the latest version of each range.
    let latest_releases: Vec<_> = {
        let mut available_ranges: Vec<_> = available
            .iter()
            .filter(|r| partial_version.map_or(true, |p| r.version.to_string().starts_with(p)))
            .map(|r| (r.version.major, r.version.minor))
            .collect();

        available_ranges.sort_unstable();
        available_ranges.dedup();

        available_ranges
            .iter()
            .filter_map(|&(major, minor)| {
                available
                    .iter()
                    .filter(|r| r.version.major == major && r.version.minor == minor)
                    .max()
            })
            .map(|r| (r, r.version.to_string()))
            .collect()
    };

    let max_len = latest_releases
        .iter()
        .map(|(_, s)| s.len())
        .max()
        .unwrap_or(0);

    let mut previous_major = None;
    let mut iter = latest_releases.iter().peekable();

    while let Some((latest, latest_string)) = iter.next() {
        let is_next_in_same_range = iter
            .peek()
            .map_or(false, |(v, _)| v.version.major == latest.version.major);

        print_list_marker(
            Some(latest.version.major) == previous_major,
            is_next_in_same_range,
        );

        previous_major = Some(latest.version.major);

        // Find all installed versions in the same range as the latest version.
        let installed_in_range: Vec<_> = installed
            .iter()
            .filter(|v| v.major == latest.version.major && v.minor == latest.version.minor)
            .copied()
            .collect();

        if installed_in_range.is_empty() {
            // No installed versions in the range.
            println!("{latest_string}");
        } else {
            let is_up_to_date = installed_in_range
                .last()
                .filter(|&v| v == &latest.version)
                .is_some()
                || installed_in_range // Special case for when installed version is newer than latest.
                    .last()
                    .map_or(false, |&v| v > latest.version);

            // Concatenate the installed versions for printing.
            let joined = installed_in_range
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ");

            if is_up_to_date {
                println!(
                    "{}",
                    format!("{latest_string:<max_len$} - Installed: {joined}").bold()
                );
            } else {
                println!(
                    "{}",
                    format!(
                        "{latest_string:<max_len$} - Installed: {joined} (update available: {})",
                        latest.date_header
                    )
                    .yellow()
                    .bold()
                );
            }
        }
    }
}

fn print_list_marker(same_as_previous: bool, same_as_next: bool) {
    print!("{} ", ranged_list_marker(same_as_previous, same_as_next));
}

fn ranged_list_marker(same_as_previous: bool, same_as_next: bool) -> &'static str {
    match (same_as_previous, same_as_next) {
        (true, true) => "├─",
        (true, false) => "└─",
        (false, true) => "┬─",
        (false, false) => "──",
    }
}
