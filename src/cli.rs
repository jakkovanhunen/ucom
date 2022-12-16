use std::fmt::{Display, Formatter};

use clap::{Args, ValueEnum};

/// Unity Commander, a command line interface for Unity projects.
#[derive(clap::Parser)]
#[command(author, version, about, arg_required_else_help = false)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Action>,
}

#[derive(clap::Subcommand)]
pub enum Action {
    #[command(verbatim_doc_comment)]
    /// This command will show a list of the installed Unity versions.
    List {
        /// The Unity versions to list. You can specify a partial version; e.g. 2021 will list all
        /// the 2021.x.y versions you have installed on your system.
        #[arg(
            short = 'u',
            long = "unity",
            value_name = "VERSION",
            verbatim_doc_comment
        )]
        version_pattern: Option<String>,
    },
    /// This command will run Unity.
    /// Unless specified otherwise, the latest installed Unity version is used.
    #[command(
        visible_alias = "r",
        verbatim_doc_comment,
        allow_hyphen_values = true,
        arg_required_else_help = true
    )]
    Run(Run),
    /// This command will create a new Unity project and Git repository in the given directory.
    /// Unless specified otherwise, the latest installed Unity version is used.
    #[command(
        verbatim_doc_comment,
        allow_hyphen_values = true,
        arg_required_else_help = true
    )]
    New(New),
    /// This command will open the Unity project in the given directory.
    #[command(
        visible_alias = "o",
        allow_hyphen_values = true,
        verbatim_doc_comment,
        arg_required_else_help = true
    )]
    Open(Open),
    /// This command will build the Unity project in the given directory.
    #[command(
        visible_alias = "b",
        allow_hyphen_values = true,
        verbatim_doc_comment,
        arg_required_else_help = true
    )]
    Build(Build),
}

#[derive(Args)]
pub struct Run {
    /// The Unity version to run. You can specify a partial version; e.g. 2021 will match the
    /// latest 2021.x.y version you have installed on your system.
    #[arg(
        short = 'u',
        long = "unity",
        value_name = "VERSION",
        verbatim_doc_comment
    )]
    pub version_pattern: Option<String>,

    /// Waits for the command to finish before continuing.
    #[clap(long = "wait", short = 'w', verbatim_doc_comment)]
    pub wait: bool,

    /// Do not print ucom log messages.
    #[clap(long = "quiet", short = 'q', verbatim_doc_comment)]
    pub quiet: bool,

    /// Show what would be run, but do not actually run it.
    #[clap(long = "dry-run", short = 'n', verbatim_doc_comment)]
    pub dry_run: bool,

    /// A list of arguments passed directly to Unity.
    #[arg(
        last = true,
        value_name = "UNITY_ARGS",
        required = true,
        verbatim_doc_comment
    )]
    pub args: Option<Vec<String>>,
}

#[derive(Args)]
pub struct New {
    /// The Unity version to use for the new project. You can specify a partial version;
    /// e.g. 2021 will match the latest 2021.x.y version you have installed on your system.
    #[arg(
        short = 'u',
        long = "unity",
        value_name = "VERSION",
        verbatim_doc_comment
    )]
    pub version_pattern: Option<String>,

    /// The directory where the project is created. This directory should not exist yet.
    #[arg(
        required = true,
        verbatim_doc_comment,
        value_name = "PROJECT_DIR",
        value_hint = clap::ValueHint::DirPath
    )]
    pub path: std::path::PathBuf,

    /// Suppress initializing a new git repository.
    #[clap(long = "no-git", verbatim_doc_comment)]
    pub no_git: bool,

    /// Waits for the command to finish before continuing.
    #[clap(long = "wait", short = 'w', verbatim_doc_comment)]
    pub wait: bool,

    /// Do not print ucom log messages.
    #[clap(long = "quiet", short = 'q', verbatim_doc_comment)]
    pub quiet: bool,

    /// Show what would be run, but do not actually run it.
    #[clap(long = "dry-run", short = 'n', verbatim_doc_comment)]
    pub dry_run: bool,

    /// A list of arguments passed directly to Unity.
    #[arg(last = true, value_name = "UNITY_ARGS", verbatim_doc_comment)]
    pub args: Option<Vec<String>>,
}

#[derive(Args)]
pub struct Open {
    /// The directory of the project.
    #[arg(value_name = "PROJECT_DIR", value_hint = clap::ValueHint::DirPath, verbatim_doc_comment)]
    pub path: std::path::PathBuf,

    /// The Unity version to open the project with. Use it to open a project with a newer
    /// Unity version. You can specify a partial version; e.g. 2021 will match the latest
    /// 2021.x.y version you have installed on your system.
    #[arg(
        short = 'u',
        long = "unity",
        value_name = "VERSION",
        verbatim_doc_comment
    )]
    pub version_pattern: Option<String>,

    /// Waits for the command to finish before continuing.
    #[clap(long = "wait", short = 'w', verbatim_doc_comment)]
    pub wait: bool,

    /// Do not print ucom log messages.
    #[clap(long = "quiet", short = 'q', verbatim_doc_comment)]
    pub quiet: bool,

    /// Show what would be run, but do not actually run it.
    #[clap(long = "dry-run", short = 'n', verbatim_doc_comment)]
    pub dry_run: bool,

    /// A list of arguments passed directly to Unity.
    #[arg(last = true, value_name = "UNITY_ARGS", verbatim_doc_comment)]
    pub args: Option<Vec<String>>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
#[allow(non_camel_case_types)]
pub enum Target {
    #[value(name = "win")]
    Win,
    #[value(name = "win64")]
    Win64,
    #[value(name = "macos")]
    OSXUniversal,
    #[value(name = "linux64")]
    Linux64,
    #[value(name = "ios")]
    iOS,
    #[value(name = "android")]
    Android,
    #[value(name = "webgl")]
    WebGL,
}

impl Display for Target {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// todo: remove?
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
#[allow(non_camel_case_types)]
pub enum BuildTarget {
    StandaloneOSX,
    StandaloneWindows,
    StandaloneWindows64,
    StandaloneLinux64,
    iOS,
    Android,
    WebGL,
}

impl Display for BuildTarget {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<Target> for BuildTarget {
    fn from(target: Target) -> Self {
        match target {
            Target::Win => Self::StandaloneWindows,
            Target::Win64 => Self::StandaloneWindows64,
            Target::OSXUniversal => Self::StandaloneOSX,
            Target::Linux64 => Self::StandaloneLinux64,
            Target::iOS => Self::iOS,
            Target::Android => Self::Android,
            Target::WebGL => Self::WebGL,
        }
    }
}

#[derive(Args)]
pub struct Build {
    /// The target platform to build for.
    #[arg(value_enum)]
    pub target: Target,

    /// The directory of the project.
    #[arg(value_name = "PROJECT_DIR", value_hint = clap::ValueHint::DirPath, verbatim_doc_comment)]
    pub path: std::path::PathBuf,

    /// The (file) path of the build output.
    #[arg(value_name = "BUILD_PATH", value_hint = clap::ValueHint::FilePath, verbatim_doc_comment)]
    pub build_path: std::path::PathBuf,

    /// Do not print ucom log messages.
    #[clap(long = "quiet", short = 'q', verbatim_doc_comment)]
    pub quiet: bool,

    /// Show what would be run, but do not actually run it.
    #[clap(long = "dry-run", short = 'n', verbatim_doc_comment)]
    pub dry_run: bool,

    /// A list of arguments passed directly to Unity.
    #[arg(last = true, value_name = "UNITY_ARGS", verbatim_doc_comment)]
    pub args: Option<Vec<String>>,
}