use std::path::Path;

use colored::Colorize;
use itertools::Itertools;

use crate::cli::PackagesInfoLevel;
use crate::unity::release_notes_url;
use crate::unity::unity_project::*;

/// Shows project information.
pub fn print_project_info(
    project_dir: &Path,
    packages_level: PackagesInfoLevel,
) -> anyhow::Result<()> {
    let project_dir = validate_project_path(&project_dir)?;
    let unity_version = version_used_by_project(&project_dir)?;

    println!(
        "{}",
        format!("Project info for: {}", project_dir.display()).bold()
    );

    let settings = ProjectSettings::from_project(&project_dir)?;
    let ps = settings.player_settings;
    println!("    Product Name:  {}", ps.product_name.bold());
    println!("    Company Name:  {}", ps.company_name.bold());
    println!("    Version:       {}", ps.bundle_version.bold());

    print!(
        "    Unity Version: {} - {}",
        unity_version.to_string().bold(),
        release_notes_url(unity_version).bright_blue()
    );

    if is_editor_installed(unity_version)? {
        println!();
    } else {
        println!(" {}", "*not installed".red().bold());
    }

    if packages_level != PackagesInfoLevel::None {
        print_project_packages(project_dir.as_ref(), packages_level)?;
    };

    Ok(())
}

/// Show packages used by the project.
fn print_project_packages(
    project_dir: &Path,
    package_level: PackagesInfoLevel,
) -> anyhow::Result<()> {
    let availability = Packages::from_project(&project_dir)?;

    match availability {
        PackagesAvailability::NoManifest => {
            println!(
                "{}",
                "No `manifest.json` file found, no packages info available.".yellow()
            );
            Ok(())
        }
        PackagesAvailability::LockFileDisabled => {
            println!(
                "{}",
                "Packages lock file is disabled in `manifest.json`, no packages info available."
                    .yellow()
            );
            Ok(())
        }
        PackagesAvailability::NoLockFile => {
            println!(
                "{}",
                "No `packages-lock.json` file found, no packages info available.".yellow()
            );
            Ok(())
        }
        PackagesAvailability::Packages(packages) => {
            let mut packages = packages
                .dependencies
                .iter()
                .filter(|(name, package)| package_level.evaluate(name, package))
                .collect_vec();

            if packages.is_empty() {
                return Ok(());
            }

            packages.sort_by(|(_, pi1), (_, pi2)| pi1.source.cmp(&pi2.source));

            println!();
            println!(
                "{} {} {}",
                "Packages:".bold(),
                package_level.to_string().bold(),
                "(L=local, E=embedded, G=git, T=tarball, R=registry, B=builtin)".bold()
            );

            for (name, package) in packages {
                println!(
                    "    {} {} ({})",
                    package.source.as_ref().map_or("?", |s| s.to_short_str()),
                    name,
                    package.version,
                );
            }
            Ok(())
        }
    }
}

impl PackagesInfoLevel {
    /// Evaluates if the `PackageInfo` is allowed by the info level.
    fn evaluate(self, name: &str, package: &PackageInfo) -> bool {
        match self {
            Self::None => false,

            Self::ExcludingUnity => {
                package.depth == 0
                    && package.source.map_or(false, |ps| {
                        ps < PackageSource::Registry
                            || (ps == PackageSource::Registry
                                && package
                                    .url
                                    .as_ref()
                                    .map_or(false, |u| u != "https://packages.unity.com"))
                    })
            }

            Self::IncludingUnity => {
                package.depth == 0
                    && package.source.map_or(false, |ps| {
                        ps < PackageSource::Builtin || name.starts_with("com.unity.feature.")
                    })
            }

            Self::All => true,
        }
    }
}
