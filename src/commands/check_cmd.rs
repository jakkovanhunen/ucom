use anyhow::anyhow;
use std::io::Write;
use std::path::Path;
use yansi::Paint;

use crate::commands::term_stat::TermStat;
use crate::commands::{writeln_b, INDENT};
use crate::unity::release_api::SortedReleases;
use crate::unity::release_api_data::ReleaseData;
use crate::unity::*;

/// Checks on the Unity website for updates to the version used by the project.
pub(crate) fn check_updates(project_dir: &Path, create_report: bool) -> anyhow::Result<()> {
    let project = ProjectPath::try_from(project_dir)?;
    let current_version = project.unity_version()?;

    let (project_updates, releases) = {
        let _status = TermStat::new("Checking", &format!("for updates to {current_version}"));
        get_latest_releases_for(current_version)?
    };

    if create_report {
        yansi::disable();
    }

    let mut buf = Vec::new();
    write_project_header(&project, create_report, &mut buf)?;
    writeln!(buf)?;

    write_project_version(
        current_version,
        project_updates,
        &releases,
        create_report,
        &mut buf,
    )?;

    if create_report {
        let download_status = TermStat::new("Downloading", "Unity release notes...");
        for release in releases.iter() {
            download_status.reprint(
                "Downloading",
                &format!("Unity {} release notes...", release.version),
            );

            write_release_notes(&mut buf, release)?;
        }
        drop(download_status);
        print!("{}", String::from_utf8(buf)?);
    } else {
        if !releases.is_empty() {
            writeln!(buf)?;
            write_available_updates(&releases, &mut buf)?;
        }
        print!("{}", String::from_utf8(buf)?);
    }

    Ok(())
}

fn write_project_header(
    project: &ProjectPath,
    create_report: bool,
    buf: &mut Vec<u8>,
) -> anyhow::Result<()> {
    if create_report {
        write!(buf, "# ")?;
    }

    writeln_b!(buf, "Unity updates for: {}", project.as_path().display())?;

    if create_report {
        writeln!(buf)?;
    }

    match ProjectSettings::from_project(project) {
        Ok(ps) => {
            writeln!(buf, "{}Product name:  {}", INDENT, ps.product_name.bold())?;
            writeln!(buf, "{}Company name:  {}", INDENT, ps.company_name.bold())?;
            writeln!(buf, "{}Version:       {}", INDENT, ps.bundle_version.bold())?;
        }

        Err(e) => {
            writeln!(
                buf,
                "{INDENT}{}: {}",
                "Could not read project settings".yellow(),
                e.yellow()
            )?;
        }
    }
    Ok(())
}

fn write_project_version(
    project_version: Version,
    project_version_info: Option<ReleaseData>,
    updates: &SortedReleases,
    create_report: bool,
    buf: &mut Vec<u8>,
) -> anyhow::Result<()> {
    let is_installed = project_version.is_editor_installed()?;
    write!(buf, "{}", "Unity editor: ".bold())?;

    let version = match (is_installed, updates.is_empty()) {
        (true, true) => {
            writeln!(buf, "{}", "installed and up to date".green().bold())?;
            project_version.green()
        }
        (true, false) => {
            writeln!(buf, "{}", "installed and out of date".yellow().bold())?;
            project_version.yellow()
        }
        (false, true) => {
            writeln!(buf, "{}", "not installed and up to date".red().bold())?;
            project_version.red()
        }
        (false, false) => {
            writeln!(buf, "{}", "not installed and out of date".red().bold())?;
            project_version.red()
        }
    };

    if create_report {
        writeln!(buf)?;
    }

    write!(
        buf,
        "{}{} - {}",
        INDENT,
        version,
        release_notes_url(project_version).bright_blue()
    )?;

    if is_installed {
        // The editor used by the project is installed, finish the line.
        writeln!(buf)?;
    } else if create_report {
        // The editor used by the project is not installed, and we're writing to a file.
        writeln!(
            buf,
            " > {}",
            project_version_info.map_or_else(
                || "No release info available".into(),
                |r| format!("[install in Unity HUB]({})", r.unity_hub_deep_link),
            )
        )?;
    } else {
        // The editor used by the project is not installed, and we're not writing to a file.
        writeln!(
            buf,
            " > {}",
            project_version_info
                .map_or_else(
                    || "No release info available".to_string(),
                    |r| r.unity_hub_deep_link.bright_blue().to_string(),
                )
                .bold()
        )?;
    }

    Ok(())
}

fn write_available_updates(releases: &SortedReleases, buf: &mut Vec<u8>) -> anyhow::Result<()> {
    writeln_b!(buf, "Available update(s):")?;
    let max_len = releases
        .iter()
        .map(|rd| rd.version.len())
        .max()
        .ok_or(anyhow!("No releases"))?;

    for release in releases.iter() {
        let status = if release.version.is_editor_installed()? {
            "installed".bold()
        } else {
            release.unity_hub_deep_link.as_str().bright_blue().bold()
        };

        writeln!(
            buf,
            "- {:<max_len$} - {} > {}",
            release.version.blue().bold(),
            release_notes_url(release.version).bright_blue(),
            status
        )?;
    }

    Ok(())
}

fn write_release_notes(buf: &mut Vec<u8>, release: &ReleaseData) -> anyhow::Result<()> {
    let url = &release.release_notes.url;
    let body = http_cache::fetch_content(url, true)?;

    writeln!(buf)?;
    writeln!(buf, "## Release notes for [{}]({url})", release.version)?;
    writeln!(buf)?;
    writeln!(
        buf,
        "[install in Unity HUB]({})",
        release.unity_hub_deep_link
    )?;

    writeln!(buf)?;
    writeln!(buf, "{}", body)?;

    Ok(())
}
