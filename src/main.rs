use anyhow::Context;
use clap::Parser;
use yansi::Paint;

use crate::cli::{Action, CacheAction, Cli};
use crate::commands::test_cmd::run_tests;
use crate::commands::*;
use crate::unity::http_cache;

mod cli;
mod cli_add;
mod cli_build;
mod cli_new;
mod cli_run;
mod cli_test;
mod commands;
mod nunit;
mod unity;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let Some(command) = cli.command else {
        return Ok(());
    };

    if cli.disable_color {
        yansi::disable();
    }

    http_cache::enable_cache_from_env()
        .with_context(|| color_error("Cannot set cache from environment"))?;

    match command {
        Action::List {
            list_type,
            version_pattern,
        } => list_versions(list_type, version_pattern.as_deref())
            .with_context(|| color_error(&format!("Cannot list `{}`", list_type))),

        Action::Info {
            project_dir,
            recursive,
            packages,
        } => project_info(&project_dir, packages, recursive)
            .with_context(|| color_error("Cannot show project info")),

        Action::Check {
            project_dir,
            report: report_path,
        } => check_updates(&project_dir, report_path)
            .with_context(|| color_error("Cannot show Unity updates for project")),

        Action::Run(settings) => run_unity(settings).context(color_error("Cannot run Unity")),

        Action::New(settings) => {
            new_project(settings).with_context(|| color_error("Cannot create new Unity project"))
        }

        Action::Open(settings) => {
            open_project(settings).with_context(|| color_error("Cannot open Unity project"))
        }

        Action::Build(settings) => {
            build_project(settings).with_context(|| color_error("Cannot build project"))
        }

        Action::Test(settings) => {
            run_tests(settings).with_context(|| color_error("Cannot run tests"))
        }

        Action::Add(arguments) => {
            add_to_project(&arguments).with_context(|| color_error("Cannot add file to project"))
        }

        Action::Cache { action: command } => {
            match command {
                CacheAction::Clear => {
                    http_cache::clear();
                    println!(
                        "Cleared cache at: {}",
                        http_cache::ucom_cache_dir().display()
                    );
                }
                CacheAction::Show => {
                    let cache_dir = http_cache::ucom_cache_dir();
                    if !cache_dir.exists() {
                        println!("No cache found at: {}", cache_dir.display());
                        return Ok(());
                    }

                    println!("Cached files at: {}", cache_dir.display());
                    for file in http_cache::ucom_cache_dir().read_dir()? {
                        println!("{}{}", INDENT, file?.file_name().to_string_lossy());
                    }
                }
            }
            Ok(())
        }
    }
}

fn color_error(message: &str) -> String {
    message.red().bold().to_string()
}
