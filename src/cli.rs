use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use clap::{Args, ValueEnum};

/// Unity Commander, a command line interface for Unity projects.
#[derive(clap::Parser)]
#[command(author, version, about, arg_required_else_help = false)]
pub struct Cli {
    /// Display the build script that is injected into the project.
    #[arg(short, short = 'I', long)]
    pub injected_script: bool,

    #[command(subcommand)]
    pub command: Option<Action>,
}

#[derive(clap::Subcommand)]
pub enum Action {
    /// Shows a list of the installed Unity versions.
    #[command(visible_alias = "l")]
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

    /// Shows project information.
    #[command(visible_alias = "i")]
    Info {
        /// The directory of the project.
        #[arg(value_name = "DIRECTORY", value_hint = clap::ValueHint::DirPath, default_value = ".")]
        project_dir: PathBuf,

        /// The level of included packages to show.
        #[arg(short, long, default_value = "more")]
        packages: PackagesInfoLevel,
    },

    /// Creates a new Unity project and Git repository (uses latest available Unity version by default)
    #[command(
        visible_alias = "n",
        allow_hyphen_values = true,
        arg_required_else_help = true
    )]
    New(NewArguments),

    /// Opens the given Unity project in the Unity Editor
    #[command(visible_alias = "o", allow_hyphen_values = true)]
    Open(OpenArguments),

    /// Builds the given Unity project
    #[command(
        visible_alias = "b",
        allow_hyphen_values = true,
        arg_required_else_help = true
    )]
    Build(BuildArguments),

    /// Runs Unity with the givens arguments (uses latest available Unity version by default)
    #[command(
        visible_alias = "r",
        allow_hyphen_values = true,
        arg_required_else_help = true
    )]
    Run(RunArguments),
}

#[derive(Args)]
pub struct RunArguments {
    /// The Unity version to run. You can specify a partial version; e.g. 2021 will match the
    /// latest 2021.x.y version you have installed on your system.
    #[arg(
        short = 'u',
        long = "unity",
        value_name = "VERSION",
        env = crate::consts::ENV_DEFAULT_VERSION,
    )]
    pub version_pattern: Option<String>,

    /// Waits for the command to finish before continuing.
    #[clap(long = "wait", short = 'w')]
    pub wait: bool,

    /// Do not print ucom log messages.
    #[clap(long = "quiet", short = 'q')]
    pub quiet: bool,

    /// Show what would be run, but do not actually run it.
    #[clap(long = "dry-run", short = 'n')]
    pub dry_run: bool,

    /// A list of arguments passed directly to Unity.
    #[arg(last = true, value_name = "UNITY_ARGS", required = true)]
    pub args: Option<Vec<String>>,
}

#[derive(Args)]
pub struct NewArguments {
    /// The Unity version to use for the new project. You can specify a partial version;
    /// e.g. 2021 will match the latest 2021.x.y version you have installed on your system.
    #[arg(
        short = 'u',
        long = "unity",
        value_name = "VERSION",
        env = crate::consts::ENV_DEFAULT_VERSION
    )]
    pub version_pattern: Option<String>,

    /// The directory where the project is created. This directory should not exist yet.
    #[arg(
        required = true,
        value_name = "DIRECTORY",
        value_hint = clap::ValueHint::DirPath
    )]
    pub project_dir: PathBuf,

    /// Suppress initializing a new git repository.
    #[clap(long = "no-git")]
    pub no_git: bool,

    /// Waits for the command to finish before continuing.
    #[clap(long = "wait", short = 'w')]
    pub wait: bool,

    /// Do not print ucom log messages.
    #[clap(long = "quiet", short = 'q')]
    pub quiet: bool,

    /// Show what would be run, but do not actually run it.
    #[clap(long = "dry-run", short = 'n')]
    pub dry_run: bool,

    /// A list of arguments passed directly to Unity.
    #[arg(last = true, value_name = "UNITY_ARGS")]
    pub args: Option<Vec<String>>,
}

#[derive(Args)]
pub struct OpenArguments {
    /// The directory of the project.
    #[arg(value_name = "DIRECTORY", value_hint = clap::ValueHint::DirPath, default_value = ".")]
    pub project_dir: PathBuf,

    /// The Unity version to open the project with. Use it to open a project with a newer
    /// Unity version. You can specify a partial version; e.g. 2021 will match the latest
    /// 2021.x.y version you have installed on your system.
    #[arg(short = 'u', long = "unity", value_name = "VERSION")]
    pub version_pattern: Option<String>,

    /// Waits for the command to finish before continuing.
    #[clap(long = "wait", short = 'w')]
    pub wait: bool,

    /// Do not print ucom log messages.
    #[clap(long = "quiet", short = 'q')]
    pub quiet: bool,

    /// Show what would be run, but do not actually run it.
    #[clap(long = "dry-run", short = 'n')]
    pub dry_run: bool,

    /// A list of arguments passed directly to Unity.
    #[arg(last = true, value_name = "UNITY_ARGS")]
    pub args: Option<Vec<String>>,
}

#[derive(Args)]
pub struct BuildArguments {
    /// The target platform to build for.
    #[arg(value_enum, env = crate::consts::ENV_BUILD_TARGET)]
    pub target: Target,

    /// The directory of the project.
    #[arg(value_name = "DIRECTORY", value_hint = clap::ValueHint::DirPath, default_value = ".")]
    pub project_dir: PathBuf,

    /// The output directory of the build. When omitted the build will be placed in <PROJECT_DIR>/Builds/<TARGET>.
    #[arg(
        short = 'o',
        long = "output",
        value_name = "DIRECTORY",
        value_hint = clap::ValueHint::FilePath
    )]
    pub build_path: Option<PathBuf>,

    /// Build script injection method.
    #[arg(
        short = 'i',
        long = "inject",
        value_name = "METHOD",
        default_value = "auto"
    )]
    pub inject: InjectAction,

    /// Build mode.
    #[arg(
        short = 'm',
        long = "mode",
        value_name = "MODE",
        default_value = "batch"
    )]
    pub mode: BuildMode,

    /// A static method in the Unity project that is called to build the project.
    #[arg(
        short = 'f',
        long = "build-function",
        value_name = "FUNCTION",
        default_value = "ucom.UcomBuilder.Build"
    )]
    pub build_function: String,

    /// The log file to write Unity's output to.
    #[arg(
        short = 'l',
        long = "log-file",
        value_name = "FILE",
        default_value = "build.log"
    )]
    pub log_file: PathBuf,

    /// Don't output the build log to stdout.
    #[clap(long = "quiet", short = 'q')]
    pub quiet: bool,

    /// Show what would be run, but do not actually run it.
    #[clap(long = "dry-run", short = 'n')]
    pub dry_run: bool,

    /// A list of arguments passed directly to Unity.
    #[arg(last = true, value_name = "UNITY_ARGS")]
    pub args: Option<Vec<String>>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum PackagesInfoLevel {
    /// Don't show any included packages.
    None,
    /// Show non-Unity packages.
    Some,
    /// + packages from the registry.
    More,
    /// + builtin packages and dependencies.
    Most,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum InjectAction {
    /// If there is no build script, inject one and remove it after the build.
    Auto,
    /// Inject the build script into the project and don't remove it afterwards.
    Persistent,
    /// Don't inject the build script and use the one that is already in the project.
    Off,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum BuildMode {
    /// Build in batch mode and wait for the build to finish.
    #[value(name = "batch")]
    Batch,

    /// Build in batch mode without the graphics device and wait for the build to finish.
    #[value(name = "batch-nogfx")]
    BatchNoGraphics,

    /// Build in the editor and quit after the build.
    #[value(name = "editor-quit")]
    EditorQuit,

    /// Build in the editor and keep it open (handy for debugging the build process).
    #[value(name = "editor")]
    Editor,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
#[allow(non_camel_case_types)]
pub enum Target {
    #[value(name = "win32")]
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
