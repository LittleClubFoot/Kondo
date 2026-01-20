use chrono::Utc;
use clap::{Parser, ValueEnum};
use kondo::{
    resolve_config_path, CollisionStrategy as LibCollisionStrategy, Config, ExecuteOptions,
    KondoOrganizer,
};

#[derive(Debug, Clone, ValueEnum)]
enum CollisionStrategy {
    /// Rename file with counter (e.g., file (1).txt)
    Rename,
    /// Skip file if it already exists
    Skip,
    /// Prompt user for each collision
    Prompt,
}

impl From<CollisionStrategy> for LibCollisionStrategy {
    fn from(strategy: CollisionStrategy) -> Self {
        match strategy {
            CollisionStrategy::Rename => LibCollisionStrategy::Rename,
            CollisionStrategy::Skip => LibCollisionStrategy::Skip,
            CollisionStrategy::Prompt => LibCollisionStrategy::Prompt,
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about = "File organizer by type")]
struct Args {
    /// Source directory to scan
    #[arg(short, long)]
    source: String,

    /// Path to the config.toml file (defaults to ~/.config/kondo/config.toml)
    #[arg(short, long)]
    config: Option<String>,

    /// How to handle file name collisions
    #[arg(long, value_enum, default_value = "rename")]
    collision: CollisionStrategy,

    /// Preview changes without moving files
    #[arg(long)]
    dry_run: bool,

    /// Process subdirectories recursively
    #[arg(short, long)]
    recursive: bool,

    /// Create backup manifest for undo capability
    #[arg(short, long)]
    backup: bool,

    /// Follow symbolic links (disabled by default for security)
    #[arg(long)]
    follow_symlinks: bool,
}

fn organize_files(args: Args) -> std::io::Result<()> {
    let source_dir = kondo::expand_tilde(&args.source);
    let config_path = resolve_config_path(args.config)?;

    if args.dry_run {
        println!("=== DRY RUN MODE: No files will be moved ===\n");
    }

    // Load config
    let config = Config::from_file(&config_path)?;

    // Check if source directory exists
    if !source_dir.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Source directory does not exist: {}", source_dir.display()),
        ));
    }

    // Create organizer
    let organizer = KondoOrganizer::new(source_dir, config);

    // Execute organization
    let options = ExecuteOptions {
        collision_strategy: args.collision.into(),
        dry_run: args.dry_run,
        recursive: args.recursive,
        backup: args.backup,
        follow_symlinks: args.follow_symlinks,
    };

    let (stats, manifest) = organizer.execute(options)?;

    // Save backup manifest if enabled
    if let Some(manifest) = manifest {
        if !manifest.operations.is_empty() && !args.dry_run {
            let manifest_path = dirs::data_local_dir()
                .ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "Could not determine local data directory",
                    )
                })?
                .join("kondo")
                .join(format!(
                    "backup-{}.json",
                    Utc::now().format("%Y%m%d-%H%M%S")
                ));

            manifest.save(&manifest_path)?;
            println!("\nBackup manifest saved to: {}", manifest_path.display());
        }
    }

    stats.print_summary(args.dry_run);
    Ok(())
}

fn main() {
    let args = Args::parse();

    if let Err(e) = organize_files(args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
