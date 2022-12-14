use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use anyhow::{anyhow, Result};
use uuid::Uuid;

use crate::cli::*;

const BUILD_SCRIPT: &str = include_str!("include/UcomBuilder.cs");
const BUILD_SCRIPT_NAME: &str = "UcomBuilder.cs";
const PERSISTENT_BUILD_SCRIPT_PATH: &str = "Assets/Plugins/ucom/Editor/UcomBuilder.cs";
const PERSISTENT_BUILD_SCRIPT_ROOT: &str = "Assets/Plugins/ucom";
const AUTO_BUILD_SCRIPT_ROOT: &str = "Assets/ucom";

type ResultFn = Box<dyn FnOnce() -> Result<()>>;

pub(crate) fn content() -> &'static str {
    BUILD_SCRIPT
}

/// Creates actions that inject a script into the project before and after the build.
pub(crate) fn new_build_script_injection_functions(
    project_dir: &Path,
    inject: InjectAction,
) -> (ResultFn, ResultFn) {
    match (
        inject,
        project_dir.join(PERSISTENT_BUILD_SCRIPT_PATH).exists(),
    ) {
        (InjectAction::Auto, true) => {
            // Build script already present, no need to inject.
            (Box::new(|| Ok(())), Box::new(|| Ok(())))
        }

        (InjectAction::Auto, false) => {
            // Build script not present, inject it.
            // Place the build script in a unique directory to avoid conflicts.
            let pre_root =
                project_dir.join(format!("{}-{}", AUTO_BUILD_SCRIPT_ROOT, Uuid::new_v4()));
            let post_root = pre_root.clone();
            (
                Box::new(|| inject_build_script(pre_root)),
                Box::new(|| remove_build_script(post_root)),
            )
        }

        (InjectAction::Persistent, true) => {
            // Build script already present, no need to inject.
            (Box::new(|| Ok(())), Box::new(|| Ok(())))
        }

        (InjectAction::Persistent, false) => {
            // Build script not present, inject it.
            let persistent_root = project_dir.join(PERSISTENT_BUILD_SCRIPT_ROOT);
            (
                Box::new(|| inject_build_script(persistent_root)),
                Box::new(|| Ok(())),
            )
        }

        (InjectAction::Off, _) => {
            // No need to do anything.
            (Box::new(|| Ok(())), Box::new(|| Ok(())))
        }
    }
}

/// Injects the build script into the project.
fn inject_build_script<P: AsRef<Path>>(parent_dir: P) -> Result<()> {
    let inject_dir = parent_dir.as_ref().join("Editor");
    fs::create_dir_all(&inject_dir)?;

    let file_path = inject_dir.join(BUILD_SCRIPT_NAME);
    println!(
        "Injecting ucom build script `{}`",
        file_path.to_string_lossy()
    );

    let mut file = File::create(file_path)?;
    write!(file, "{}", BUILD_SCRIPT).map_err(|e| e.into())
}

/// Removes the injected build script from the project.
fn remove_build_script<P: AsRef<Path>>(parent_dir: P) -> Result<()> {
    if !parent_dir.as_ref().exists() {
        return Ok(());
    }

    println!(
        "Removing injected ucom build script in directory `{}`",
        parent_dir.as_ref().to_string_lossy()
    );

    // Remove the directory where the build script is located.
    fs::remove_dir_all(&parent_dir).map_err(|_| {
        anyhow!(
            "Could not remove directory `{}`",
            parent_dir.as_ref().to_string_lossy()
        )
    })?;

    // Remove the .meta file.
    let meta_file = parent_dir.as_ref().with_extension("meta");
    if !meta_file.exists() {
        return Ok(());
    }

    fs::remove_file(&meta_file)
        .map_err(|_| anyhow!("Could not remove `{}`", meta_file.to_string_lossy()))
}
