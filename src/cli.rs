use std::path::PathBuf;

use clap::{Args, ValueEnum};
use strum::Display;

use crate::cli_add::AddArguments;
use crate::cli_build::{BuildArguments, OpenTarget};
use crate::cli_new::NewArguments;
use crate::cli_run::RunArguments;
use crate::cli_test::TestArguments;

pub const ENV_EDITOR_DIR: &str = "UCOM_EDITOR_DIR";
pub const ENV_BUILD_TARGET: &str = "UCOM_BUILD_TARGET";
pub const ENV_PACKAGE_LEVEL: &str = "UCOM_PACKAGE_LEVEL";

/// Unity Commander: A command-line interface for Unity projects.
#[derive(clap::Parser)]
#[command(author, version, about)]
#[command(arg_required_else_help = true)]
pub struct Cli {
    /// Disables colored output.
    #[arg(long, short = 'D')]
    pub disable_color: bool,

    #[command(subcommand)]
    pub command: Option<Action>,
}

#[derive(clap::Subcommand)]
pub enum Action {
    /// Lists installed Unity versions.
    #[command(visible_alias = "l")]
    List {
        /// Defines what to list.
        #[arg(value_enum, default_value = "installed")]
        list_type: ListType,

        /// Filter Unity versions by prefix (e.g. '2021' shows all 2021.x.x versions)
        #[arg(short = 'u', long = "unity", value_name = "VERSION")]
        version_filter: Option<String>,

        /// Forces downloading release data from the Unity API.
        #[arg(short = 'f', long)]
        force: bool,
    },

    /// Installs the latest Unity version that matches the specified version range.
    #[command(visible_alias = "I")]
    Install {
        /// The version of Unity to install. Can be a version prefix like '2023.1' or a full version like '2021.1.0f1'.
        #[arg(value_name = "VERSION")]
        version: String,
    },

    /// Displays project information.
    #[command(visible_alias = "i")]
    Info {
        /// Specifies the project's directory.
        #[arg(value_name = "DIRECTORY", value_hint = clap::ValueHint::DirPath, default_value = ".")]
        project_dir: PathBuf,

        /// Installs the required Unity version if it is not installed.
        #[arg(short = 'I', long)]
        install: bool,

        /// Recursively searches for Unity projects in the given directory.
        #[arg(short = 'R', long)]
        recursive: bool,

        /// Determines the level of package information to display.
        #[arg(short = 'p', long, default_value = "no-unity", env = ENV_PACKAGE_LEVEL)]
        packages: PackagesInfoLevel,
    },

    /// Checks for Unity updates for the project.
    #[command(visible_alias = "c")]
    Check {
        /// Specifies the project's directory.
        #[arg(value_name = "DIRECTORY", value_hint = clap::ValueHint::DirPath, default_value = ".")]
        project_dir: PathBuf,

        /// Installs the latest Unity version if it is not installed.
        #[arg(short = 'I', long)]
        install: bool,

        /// Generates a Markdown report of aggregated release notes.
        #[arg(short = 'r', long)]
        report: bool,
    },

    /// Creates a new Unity project and Git repository, defaulting to the latest-installed Unity version.
    #[command(visible_alias = "n")]
    New(NewArguments),

    /// Opens a specified Unity project in the Unity Editor.
    #[command(visible_alias = "o")]
    Open(OpenArguments),

    /// Builds a specified Unity project.
    #[command(visible_alias = "b")]
    Build(BuildArguments),

    /// Runs tests in the Project.
    #[command(visible_alias = "t")]
    Test(TestArguments),

    /// Runs Unity with specified arguments, defaulting to the latest-installed Unity version.
    #[command(visible_alias = "r")]
    Run(RunArguments),

    /// Adds a helper script or configuration file to the project.
    Add(AddArguments),

    /// Handles caching for downloaded Unity release data.
    ///
    /// By default, cached files have a lifespan of one hour.
    /// After this time, the system will re-download the required files for updated data.
    ///
    /// Use the `UCOM_ENABLE_CACHE` environment variable to control caching.
    /// Set it to `false` if you want to disable the download cache feature.
    /// When disabled, the system will download the required Unity release data afresh
    /// for every command, instead of using cached files.
    #[command()]
    Cache {
        #[arg(value_enum)]
        action: CacheAction,
    },
}

#[derive(Args)]
pub struct OpenArguments {
    /// Specifies the project's directory.
    #[arg(value_name = "DIRECTORY", value_hint = clap::ValueHint::DirPath, default_value = ".")]
    pub project_dir: PathBuf,

    /// Upgrades the project's Unity version.
    /// If no version is specified, it upgrades to the latest available version within the project's `major.minor` range.
    /// A version prefix like '2021' selects the latest-installed version within the 2021.x.y range.
    #[arg(short = 'U', long = "upgrade", value_name = "VERSION")]
    pub upgrade_version: Option<Option<String>>,

    /// Determines the active build target to open the project with.
    #[arg(short = 't', long, value_name = "NAME")]
    pub target: Option<OpenTarget>,

    /// Waits for the command to complete before proceeding.
    #[arg(short = 'w', long)]
    pub wait: bool,

    /// Closes the editor after opening the project.
    #[arg(short = 'Q', long)]
    pub quit: bool,

    /// Suppresses ucom messages.
    #[arg(short = 'q', long)]
    pub quiet: bool,

    /// Shows the command to be run without actually executing it.
    #[arg(short = 'n', long)]
    pub dry_run: bool,

    /// A list of arguments to be passed directly to Unity.
    #[arg(last = true, value_name = "UNITY_ARGS")]
    pub args: Option<Vec<String>>,
}

#[derive(Debug, Display, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ListType {
    /// Lists the installed Unity versions.
    Installed,
    /// Displays installed Unity versions and checks for online updates.
    Updates,
    /// Shows the latest available Unity versions.
    Latest,
    /// Shows all available Unity versions.
    All,
}

#[derive(Display, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum PackagesInfoLevel {
    /// No package information is displayed.
    #[value(name = "none")]
    #[strum(serialize = "none")]
    None,

    /// Shows non-Unity packages only.
    #[value(name = "no-unity")]
    #[strum(serialize = "no-unity")]
    ExcludingUnity,

    /// Additionally includes information for packages from the Unity registry.
    #[value(name = "inc-unity")]
    #[strum(serialize = "inc-unity")]
    IncludingUnity,

    /// Displays all package information including built-in packages and dependencies.
    #[value(name = "all")]
    #[strum(serialize = "all")]
    All,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum CacheAction {
    /// Removes all files from the cache.
    Clear,
    /// Displays a list of all currently cached files.
    List,
}
