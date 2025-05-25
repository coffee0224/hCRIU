# hCRIU

hCRIU is a tool to creat and restore checkpoint of process based on [CRIU](https://github.com/checkpoint-restore/criu). It support checkpoint manager, including periodically creat, merge and restore.

## How to use
Firstly, you need to install [criu](https://criu.org/Packages).

### Create a checkpoint
```shell
# Basic checkpoint creation
hcriu dump <PID>

# Create checkpoint with a tag
hcriu dump <PID> --tag my-checkpoint

# Create periodic checkpoints (e.g., every 10 seconds)
hcriu dump <PID> --interval 10s

# Create checkpoint and leave the process running
hcriu dump <PID> --leave-running
```

### Restore from checkpoint
```shell
# Restore from a specific checkpoint
hcriu restore <checkpoint-id>
```

### List checkpoints
```shell
# List all checkpoints (sorted by time by default)
hcriu list

# List checkpoints sorted by PID
hcriu list --sort pid
```

### Merge checkpoints
```shell
# Merge checkpoints with a specific tag
hcriu merge <tag>

# Merge checkpoints for a specific process
hcriu merge <tag> --pid <PID>

# Keep daily checkpoints while merging
hcriu merge <tag> --keep-daily

# Keep hourly checkpoints while merging
hcriu merge <tag> --keep-hourly

# Dry run to see what would be merged
hcriu merge <tag> --dry-run
```

### Additional Options
- `--criu-path`: Specify custom CRIU executable path (default find by which)
- `-D, --hcriu-dir`: Specify checkpoints directory (default: ~/.hcriu/)

## Useful Link

- [Checkpoint/Restore - CRIU](https://criu.org/Checkpoint/Restore)
- [C API - CRIU](https://criu.org/C_API)
- [RPC - CRIU](https://criu.org/RPC)
- [rust_criu - Rust](https://docs.rs/rust-criu/latest/rust_criu/)
