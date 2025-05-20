use clap::{Parser, Subcommand, ValueEnum};
use humantime::Duration;
use bytesize::ByteSize;

#[derive(Debug, ValueEnum, Clone)]
enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Parser)]
#[command(name = "hCRIU")]
#[command(about = "checkpoint management tool")]
struct Cli {
    /// Specify configuration file path
    #[arg(short, long, default_value = "~/.checkpointctl.yaml")]
    config: String,

    /// Log level
    #[arg(long, default_value = "info")]
    log_level: LogLevel,

    /// Specify CRIU executable path
    #[arg(long)]
    criu_path: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Create a new checkpoint
    Create {
        /// Container ID
        container_id: String,

        /// Create interval (e.g., 30m, 1h)
        #[arg(short, long)]
        interval: Option<Duration>,

        /// Add metadata label (can be used multiple times)
        #[arg(short, long)]
        label: Vec<String>,

        /// Parent checkpoint ID (incremental mode)
        #[arg(short, long)]
        parent: Option<String>,

        /// Compression algorithm
        #[arg(long, default_value = "zstd")]
        compression: CompressionAlgorithm,

        /// Freeze container processes before creation
        #[arg(long)]
        pre_freeze: bool,

        /// Memory snapshot limit (e.g., 512MB)
        #[arg(long, default_value = "1G")]
        memory_limit: ByteSize,
    },

    /// Restore container from checkpoint
    Restore {
        /// Checkpoint ID
        checkpoint_id: String,

        /// Target node for restoration
        #[arg(short, long)]
        target_node: Option<String>,

        /// Number of parallel restore threads
        #[arg(long, default_value = "3")]
        parallel: usize,

        /// Validate data integrity before restore
        #[arg(long)]
        validate: bool,

        /// Automatically resume process execution state
        #[arg(long)]
        resume: bool,

        /// Remap container network (e.g., 10.0.0.0/24→192.168.0.0/24)
        #[arg(long)]
        network_remap: Option<String>,
    },

    /// List all checkpoints
    List {
        /// Container ID (optional)
        container_id: Option<String>,

        /// Output format
        #[arg(short, long)]
        output: Option<OutputFormat>,

        /// Sort field
        #[arg(long)]
        sort: Option<SortField>,

        /// Filter expression
        #[arg(short, long)]
        filter: Option<String>,

        /// Show checkpoint dependency tree
        #[arg(long)]
        show_tree: bool,
    },

    /// Merge multiple checkpoints
    Merge {
        /// Container ID
        container_id: String,

        /// Merge strategy
        #[arg(short, long)]
        strategy: MergeStrategy,

        /// Keep N days of daily checkpoints
        #[arg(long)]
        keep_daily: Option<usize>,

        /// Keep N hours of hourly checkpoints
        #[arg(long)]
        keep_hourly: Option<usize>,

        /// Retain checkpoints with specified labels
        #[arg(long)]
        retain_labels: Vec<String>,

        /// Enable deep deduplication mode
        #[arg(long)]
        aggressive: bool,

        /// Simulate merge without actual execution
        #[arg(long)]
        dry_run: bool,
    },

    /// Clean up old checkpoints
    Prune {
        /// Container ID
        container_id: String,

        /// Keep N latest checkpoints
        #[arg(long)]
        keep_latest: Option<usize>,

        /// Keep checkpoints within N days
        #[arg(long)]
        keep_days: Option<usize>,

        /// Exclude checkpoints with specified labels
        #[arg(long)]
        exclude_labels: Vec<String>,

        /// Storage space limit
        #[arg(long)]
        max_storage: Option<ByteSize>,

        /// Clean up orphaned checkpoints
        #[arg(long)]
        prune_dangling: bool,
    },

    /// Show detailed information
    Info {
        /// Checkpoint ID
        checkpoint_id: String,

        /// Show complete metadata
        #[arg(long)]
        metadata: bool,

        /// Compare with another checkpoint
        #[arg(long)]
        diff: Option<String>,

        /// Verify data integrity
        #[arg(long)]
        verify: bool,

        /// Export to portable format
        #[arg(long)]
        export: bool,
    },

    /// Automatic management daemon
    Automanage {
        /// Automatic check interval
        #[arg(long, default_value = "5m")]
        check_interval: Duration,

        /// CPU threshold for creation trigger
        #[arg(long, default_value = "80%")]
        cpu_threshold: String,

        /// Memory usage threshold
        #[arg(long, default_value = "90%")]
        mem_threshold: String,

        /// Custom policy configuration file path
        #[arg(long)]
        schedule_file: Option<String>,

        /// Run as daemon
        #[arg(long)]
        daemonize: bool,
    },
}

#[derive(Debug, ValueEnum, Clone)]
enum CompressionAlgorithm {
    None,
    Gzip,
    Zstd,
}

#[derive(Debug, ValueEnum, Clone)]
enum OutputFormat {
    Text,
    Json,
    Yaml,
}

#[derive(Debug, ValueEnum, Clone)]
enum SortField {
    Time,
    Size,
    Labels,
}

#[derive(Debug, ValueEnum, Clone)]
enum MergeStrategy {
    TimeBased,
    Incremental,
    Tagged,
}

fn main() {
    let cli = Cli::parse();
    println!("{:?}", cli);
}
