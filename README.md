# hCRIU

hCRIU is a tool to creat and restore checkpoint of process based on [CRIU](https://github.com/checkpoint-restore/criu). It support checkpoint manager, including periodically creat, merge and restore.

## How to build
Firstly, clone the project.

Second, you need to install [criu](https://criu.org/Packages).  Or if you have [nix](https://nixos.org/download/) environment, use `nix develop .` to run into develop shell including [criu](https://criu.org/Packages). use `exit` command to exit the develop shell.

```shell
cargo build
```

the `hcriu` executable file is under `./target/debug/hCRIU`, you can `alias hcriu=./target/debug/hCRIU`, or `mv` it outside, whatever you like

use `hcriu --help` to see command option

## How to use

### Create a checkpoint
```shell
# Basic checkpoint creation
hcriu [--tui] dump <PID>

# Create checkpoint with a tag
hcriu [--tui] dump <PID> --tag my-checkpoint

# Create periodic checkpoints (e.g., every 10 seconds)
hcriu [--tui] dump <PID> --interval 10s

# Create checkpoint and leave the process running
hcriu [--tui] dump <PID> --leave-running
```

### Restore from checkpoint
```shell
# Restore from a specific checkpoint
hcriu restore <checkpoint-id>
```

### List checkpoints
```shell
# List all checkpoints (sorted by time by default)
hcriu [--tui] list

# List checkpoints sorted by PID
hcriu [--tui] list --sort pid
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
- `--help`: Show cli options

### TUI Mode
```shell
hcriu --tui
```

press `e` to enter command, for example: `list`. this flow are shown in ppt, check it as you like.

## Useful Link

- [Checkpoint/Restore - CRIU](https://criu.org/Checkpoint/Restore)
- [C API - CRIU](https://criu.org/C_API)
- [RPC - CRIU](https://criu.org/RPC)
- [rust_criu - Rust](https://docs.rs/rust-criu/latest/rust_criu/)
