use crate::unity::Version;
use crate::utils::vec1::{Vec1, Vec1Error};
use anyhow::{Context, anyhow};
use itertools::Itertools;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::{env, fs};

const ENV_EDITOR_DIR: &str = "UCOM_EDITOR_DIR";

/// Sub path to the executable on macOS.
#[cfg(target_os = "macos")]
const UNITY_EDITOR_EXE: &str = "Unity.app/Contents/MacOS/Unity";

/// Sub path to the executable on Windows.
#[cfg(target_os = "windows")]
const UNITY_EDITOR_EXE: &str = r"Editor\Unity.exe";

/// Other target platforms are not supported.
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const UNITY_EDITOR_EXE: &str = compile_error!("Unsupported platform");

/// Parent directory of editor installations on macOS.
#[cfg(target_os = "macos")]
const UNITY_EDITOR_DIR: &str = "/Applications/Unity/Hub/Editor/";

/// Parent directory of editor installations on Windows.
#[cfg(target_os = "windows")]
const UNITY_EDITOR_DIR: &str = r"C:\Program Files\Unity\Hub\Editor";

/// Other target platforms are not supported.
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const UNITY_EDITOR_DIR: &str = compile_error!("Unsupported platform");

//
// VersionList
//

/// A non-empty list of Unity versions, sorted from the oldest to the newest.
pub struct VersionList(Vec1<Version>);

impl Deref for VersionList {
    type Target = Vec1<Version>;

    /// Returns a reference to the inner [`Vec1`].
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<Vec<Version>> for VersionList {
    type Error = Vec1Error;

    /// Converts a vector of versions into a sorted list of versions.
    fn try_from(versions: Vec<Version>) -> Result<Self, Self::Error> {
        Vec1::try_from(versions).map(|mut versions| {
            versions.sort_unstable();
            Self(versions)
        })
    }
}

impl VersionList {
    /// Converts the list into a [`Vec`].
    pub fn into_vec(self) -> Vec<Version> {
        self.0.into()
    }

    /// Returns a sorted list of installed Unity versions from the given directory or an error if no versions are found.
    fn from_dir(dir: impl AsRef<Path>) -> anyhow::Result<Self> {
        let versions = fs::read_dir(&dir)
            .with_context(|| {
                format!(
                    "Cannot read available Unity editors in `{}`",
                    dir.as_ref().display()
                )
            })?
            .map_while(Result::ok)
            .map(|de| de.path()) //
            .filter(|p| p.is_dir() && p.join(UNITY_EDITOR_EXE).exists())
            .filter_map(|p| p.file_name()?.to_string_lossy().parse::<Version>().ok())
            .collect_vec();

        Self::try_from(versions).map_err(|_| {
            anyhow!(
                "No Unity installations found in `{}`",
                dir.as_ref().display()
            )
        })
    }

    pub fn filter_by_prefix(self, version_prefix: Option<&str>) -> anyhow::Result<Self> {
        let Some(version_prefix) = version_prefix else {
            // No version to match, return the full list again.
            return Ok(self);
        };

        let mut versions = self.into_vec();
        versions.retain(|v| v.as_str().starts_with(version_prefix));

        Vec1::try_from(versions).map(VersionList).map_err(|_| {
            anyhow!("No Unity installation was found that matches version `{version_prefix}`.")
        })
    }
}

//
// Installations
//

/// The installed versions and the root directory they are installed in.
pub struct Installations {
    pub install_dir: PathBuf,
    pub versions: VersionList,
}

impl Installations {
    /// Returns a list of installed Unity versions or an error if no versions are found.
    pub fn find_installations(version_prefix: Option<&str>) -> anyhow::Result<Self> {
        let install_dir = Self::editor_parent_dir()?.to_path_buf();
        let versions = VersionList::from_dir(&install_dir)?.filter_by_prefix(version_prefix)?;
        Ok(Self {
            install_dir,
            versions,
        })
    }

    /// Returns a list of installed Unity versions or `None` if no versions are found.
    pub fn try_find_installations(version_prefix: Option<&str>) -> Option<Self> {
        Self::find_installations(version_prefix).ok()
    }

    /// Returns the version of the latest-installed version that matches the given prefix.
    pub fn latest_installed_version(version_prefix: Option<&str>) -> anyhow::Result<Version> {
        let version = *VersionList::from_dir(Self::editor_parent_dir()?)?
            .filter_by_prefix(version_prefix)?
            .last();
        Ok(version)
    }

    /// Returns the parent directory of the editor installations.
    fn editor_parent_dir() -> anyhow::Result<&'static Path> {
        static EDITOR_PARENT_DIR: OnceLock<PathBuf> = OnceLock::new();

        if let Some(path) = EDITOR_PARENT_DIR.get() {
            return Ok(path);
        }

        if let Ok(path) = Self::resolve_unity_editor_directory() {
            Self::verify_directory_contains_unity_installations(&path)?;
            Ok(EDITOR_PARENT_DIR.get_or_init(|| path))
        } else {
            Err(Self::create_unity_installation_not_found_error())
        }
    }

    /// Resolves the Unity editor directory from the environment variable or the default path.
    fn resolve_unity_editor_directory() -> anyhow::Result<PathBuf> {
        // Try to get the directory from the environment variable.
        let path = if let Some(path) = env::var_os(ENV_EDITOR_DIR) {
            // Use the directory set by the environment variable.
            let path = Path::new(&path);
            // If the directory does not exist or is not a directory, return an error.
            if !path.is_dir() {
                return Err(anyhow!(
                    "Editor directory set by `{ENV_EDITOR_DIR}` is not a valid directory: `{}`",
                    path.display()
                ));
            }
            path.to_owned()
        } else {
            // Use the default directory.
            let path = Path::new(UNITY_EDITOR_DIR);
            if !path.is_dir() {
                return Err(anyhow!(
                    "The default editor directory `{UNITY_EDITOR_DIR}` is not a valid directory`"
                ));
            }
            path.to_owned()
        };
        Ok(path)
    }

    /// Check if the directory contains Unity installations.
    fn verify_directory_contains_unity_installations(dir: impl AsRef<Path>) -> anyhow::Result<()> {
        let dir = dir.as_ref();
        if dir
            .read_dir()
            .with_context(|| format!("Cannot read editor directory `{}`", dir.display()))?
            .any(|entry| {
                entry
                    .as_ref()
                    .map(|e| e.path().join(UNITY_EDITOR_EXE).exists())
                    .unwrap_or(false)
            })
        {
            return Ok(());
        }
        Err(Self::create_unity_installation_not_found_error())
    }

    /// Creates an error indicating that no Unity installations were found.
    fn create_unity_installation_not_found_error() -> anyhow::Error {
        // Could not find any installations, check if the editor directory is set.
        match env::var_os(ENV_EDITOR_DIR) {
            Some(path) => {
                if Path::new(&path).is_dir() {
                    // The editor directory is set, but does not contain any installations.
                    anyhow!(
                        "No Unity installations found in the editor directory `{}`. Please set the `{ENV_EDITOR_DIR}` environment variable to the correct path.",
                        path.to_string_lossy()
                    )
                } else {
                    // The editor directory is set,but it is not valid.
                    anyhow!(
                        "The editor directory set by `{ENV_EDITOR_DIR}` is not valid: `{}`",
                        path.to_string_lossy()
                    )
                }
            }
            None => {
                // The editor directory is not set and no installations were found.
                anyhow!(
                    "No Unity installations found in the default directory `{}`. Please set the `{ENV_EDITOR_DIR}` environment variable to the correct path.",
                    UNITY_EDITOR_DIR
                )
            }
        }
    }
}

impl Version {
    /// Returns true if the editor is installed.
    pub fn is_editor_installed(self) -> anyhow::Result<bool> {
        Ok(Installations::editor_parent_dir()?
            .join(self.as_str())
            .exists())
    }

    /// Returns the path to the editor executable.
    pub fn editor_executable_path(self) -> anyhow::Result<PathBuf> {
        let exe_path = Installations::editor_parent_dir()?
            .join(self.as_str())
            .join(UNITY_EDITOR_EXE);

        if exe_path.exists() {
            Ok(exe_path)
        } else {
            Err(anyhow!("Unity version is not installed: {self}"))
        }
    }
}
