use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use firecracker::runtime::bundled::{BundledMode, BundledRuntimeOptions};

#[derive(Debug, Parser)]
#[command(
    name = "fc-cli",
    version,
    about = "CLI utilities for Firecracker SDK runtime operations"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Resolve firecracker/jailer binaries using bundled runtime rules.
    Resolve(ResolveArgs),
    /// Print current platform and whether release-based bundled mode supports it.
    Platform,
}

#[derive(Debug, Clone, Args)]
struct ResolveArgs {
    /// Which binary path to resolve.
    #[arg(value_enum, default_value_t = ResolveTarget::All)]
    target: ResolveTarget,

    /// Binary resolution mode.
    #[arg(long, value_enum, default_value_t = ResolveMode::BundledThenSystem)]
    mode: ResolveMode,

    /// Root directory containing bundled artifacts.
    #[arg(long)]
    bundle_root: Option<PathBuf>,

    /// Firecracker release version (e.g., v1.12.1).
    #[arg(long)]
    release: Option<String>,

    /// Expected SHA256 for firecracker binary.
    #[arg(long)]
    firecracker_sha256: Option<String>,

    /// Expected SHA256 for jailer binary.
    #[arg(long)]
    jailer_sha256: Option<String>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ResolveTarget {
    Firecracker,
    Jailer,
    All,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ResolveMode {
    BundledOnly,
    SystemOnly,
    BundledThenSystem,
    SystemThenBundled,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Resolve(args) => resolve(args)?,
        Commands::Platform => platform(),
    }
    Ok(())
}

fn resolve(args: ResolveArgs) -> Result<(), Box<dyn std::error::Error>> {
    let mut opts = BundledRuntimeOptions::new().mode(to_bundled_mode(args.mode));

    if let Some(bundle_root) = args.bundle_root {
        opts = opts.bundle_root(bundle_root);
    }
    if let Some(release) = args.release {
        opts = opts.release_version(release);
    }
    if let Some(sha) = args.firecracker_sha256 {
        opts = opts.firecracker_sha256(sha);
    }
    if let Some(sha) = args.jailer_sha256 {
        opts = opts.jailer_sha256(sha);
    }

    match args.target {
        ResolveTarget::Firecracker => {
            let path = opts.resolve_firecracker_bin()?;
            println!("firecracker={}", path.display());
        }
        ResolveTarget::Jailer => {
            let path = opts.resolve_jailer_bin()?;
            println!("jailer={}", path.display());
        }
        ResolveTarget::All => {
            let firecracker = opts.resolve_firecracker_bin()?;
            let jailer = opts.resolve_jailer_bin()?;
            println!("firecracker={}", firecracker.display());
            println!("jailer={}", jailer.display());
        }
    }

    Ok(())
}

fn platform() {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let supported = os == "linux" && (arch == "x86_64" || arch == "aarch64");

    println!("os={os}");
    println!("arch={arch}");
    println!("bundled_release_supported={supported}");
}

fn to_bundled_mode(mode: ResolveMode) -> BundledMode {
    match mode {
        ResolveMode::BundledOnly => BundledMode::BundledOnly,
        ResolveMode::SystemOnly => BundledMode::SystemOnly,
        ResolveMode::BundledThenSystem => BundledMode::BundledThenSystem,
        ResolveMode::SystemThenBundled => BundledMode::SystemThenBundled,
    }
}
