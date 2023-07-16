use std::process::Command;

use colored::Colorize;

use crate::cli::*;
use crate::unity::*;

/// Opens the given Unity project in the Unity Editor.
pub fn open_project(arguments: OpenArguments) -> anyhow::Result<()> {
    let project_dir = validate_project_path(&arguments.project_dir)?;

    let project_unity_version = version_used_by_project(&project_dir)?;

    let open_unity_version = match arguments.version_pattern {
        Some(Some(pattern)) => matching_available_version(Some(&pattern)),
        Some(None) => matching_available_version(Some(&project_unity_version.minor_partial())),
        None => Ok(project_unity_version),
    }?;

    let editor_exe = editor_executable_path(open_unity_version)?;

    check_for_assets_directory(&project_dir)?;

    // Build the command to execute.
    let mut cmd = Command::new(editor_exe);
    cmd.args(["-projectPath", &project_dir.to_string_lossy()]);

    if let Some(target) = arguments.target {
        cmd.args(["-buildTarget", &target.to_string()]);
    }

    if arguments.quit {
        cmd.arg("-quit");
    }

    cmd.args(arguments.args.unwrap_or_default());

    if arguments.dry_run {
        println!("{}", cmd_to_string(&cmd));
        return Ok(());
    }

    if !arguments.quiet {
        println!(
            "{}",
            format!(
                "Open Unity {} project in: {}",
                open_unity_version,
                project_dir.display()
            )
            .bold()
        );
    }

    if arguments.wait {
        wait_with_stdout(cmd)?;
    } else {
        spawn_and_forget(cmd)?;
    }
    Ok(())
}
