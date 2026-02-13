# Firecracker Rust Client SDK

Rust SDK for the [Firecracker](https://github.com/firecracker-microvm/firecracker) microVM API.

## Crates

| Crate | Description |
|---|---|
| `firecracker` | Facade — re-exports `api` + `sdk` + optional `runtime` |
| `fc-api` | Low-level typed client, generated from Swagger spec via [progenitor](https://github.com/oxidecomputer/progenitor) |
| `fc-sdk` | High-level typestate wrapper: `VmBuilder` → `Vm` lifecycle, process management |
| `fc-cli` | CLI for runtime helpers and microVM process launch |

## Quick Start

```rust
use firecracker::sdk::{VmBuilder, types::*};

let vm = VmBuilder::new("/tmp/firecracker.socket")
    .boot_source(BootSource {
        kernel_image_path: "/path/to/vmlinux".into(),
        boot_args: Some("console=ttyS0".into()),
        initrd_path: None,
    })
    .machine_config(MachineConfiguration {
        vcpu_count: std::num::NonZeroU64::new(2).unwrap(),
        mem_size_mib: 256,
        smt: false,
        track_dirty_pages: false,
        cpu_template: None,
        huge_pages: None,
    })
    .start()
    .await?;

vm.pause().await?;
vm.resume().await?;
vm.create_snapshot("/tmp/snap", "/tmp/mem").await?;
```

For operations not covered by the SDK, use the generated API client directly:

```rust
let client = vm.client();
client.describe_instance().send().await?;
```

## Process Management

`fc-sdk` can spawn and manage the Firecracker process for you.

### Direct (no jailer)

```rust
use firecracker::sdk::FirecrackerProcessBuilder;

let process = FirecrackerProcessBuilder::new("/usr/bin/firecracker", "/tmp/fc.sock")
    .id("my-vm")
    .no_seccomp(true)
    .cleanup_socket(true)
    .spawn()
    .await?;

let vm = process.vm_builder()
    .boot_source(/* ... */)
    .machine_config(/* ... */)
    .start()
    .await?;

// process is killed and socket cleaned up on drop
```

### Via Jailer

```rust
use firecracker::sdk::JailerProcessBuilder;

let process = JailerProcessBuilder::new(
    "/usr/bin/jailer",
    "/usr/bin/firecracker",
    "my-vm",
    1000, // uid
    1000, // gid
)
.netns("/var/run/netns/my-ns")
.spawn()
.await?;

// Socket path is automatically computed from the chroot layout
let vm = process.vm_builder()
    .boot_source(/* ... */)
    .machine_config(/* ... */)
    .start()
    .await?;
```

### Restoring from Snapshot

```rust
use firecracker::sdk::{restore, types::*};

let vm = restore(
    "/tmp/firecracker.sock",
    SnapshotLoadParams {
        snapshot_path: "/path/to/snapshot".into(),
        mem_file_path: Some("/path/to/mem".into()),
        mem_backend: None,
        enable_diff_snapshots: None,
        track_dirty_pages: None,
        resume_vm: Some(true),
        network_overrides: vec![],
    },
).await?;
```

### Rebuilding from Exported Config

```rust
// Export config from a running VM
let config = vm.config().await?;

// Recreate a builder on a new Firecracker instance
let new_vm = VmBuilder::from_config("/tmp/new-fc.sock", config)
    .mmds_data(/* optional pre-boot MMDS data */)
    .start()
    .await?;
```

## Bundled Runtime Mode

Enable this capability with:

```bash
cargo add firecracker --features bundled-runtime
```

`firecracker::runtime::bundled` supports resolving `firecracker` and `jailer` binaries from bundled
Firecracker upstream release artifacts with optional fallback to system `PATH`.
Release-based bundled mode currently supports:

- `linux-x86_64`
- `linux-aarch64`

```rust
use firecracker::runtime::bundled::{BundledMode, BundledRuntimeOptions};

let bundled = BundledRuntimeOptions::new()
    .mode(BundledMode::BundledThenSystem)
    .bundle_root("/opt/arcbox/bundled")
    .release_version("v1.12.1")
    .firecracker_sha256("sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
    .jailer_sha256("sha256:abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789");

let fc = bundled.firecracker_builder("/tmp/firecracker.socket")?;

let jailer = bundled.jailer_builder("vm-1", 1000, 1000)?;
```

Supported bundled path layout:

- `{bundle_root}/release-vX.Y.Z-{arch}/firecracker-vX.Y.Z-{arch}`
- `{bundle_root}/release-vX.Y.Z-{arch}/jailer-vX.Y.Z-{arch}`
- `{bundle_root}/firecracker-vX.Y.Z-{arch}`
- `{bundle_root}/jailer-vX.Y.Z-{arch}`
- `{bundle_root}/{os}-{arch}/{binary}`
- `{bundle_root}/{os}-{arch}/bin/{binary}`
- `{bundle_root}/{arch}-{os}/{binary}`
- `{bundle_root}/{arch}-{os}/bin/{binary}`
- `{bundle_root}/{binary}`

Environment variable overrides:

- `FC_SDK_FIRECRACKER_BIN`
- `FC_SDK_JAILER_BIN`
- `FC_SDK_BUNDLED_DIR`
- `FC_SDK_FIRECRACKER_RELEASE`

`fc-cli` usage examples:

```bash
# Resolve both binaries using bundled-first strategy
cargo run -p fc-cli -- resolve all --mode bundled-then-system --bundle-root /opt/arcbox/bundled --release v1.12.1

# Resolve only firecracker from system PATH
cargo run -p fc-cli -- resolve firecracker --mode system-only

# Start a microVM and keep fc-cli attached (Ctrl+C to stop)
cargo run -p fc-cli -- start \
  --mode bundled-then-system \
  --bundle-root /opt/arcbox/bundled \
  --release v1.12.1 \
  --kernel /path/to/vmlinux \
  --rootfs /path/to/rootfs.ext4 \
  --socket-path /tmp/firecracker.socket

# Start a microVM in detached mode
cargo run -p fc-cli -- start \
  --kernel /path/to/vmlinux \
  --rootfs /path/to/rootfs.ext4 \
  --detach

# Start a microVM via jailer in detached mode
cargo run -p fc-cli -- start \
  --backend jailer \
  --uid 1000 \
  --gid 1000 \
  --kernel /path/to/vmlinux \
  --rootfs /path/to/rootfs.ext4 \
  --detach

# Show platform support for release-based bundled mode
cargo run -p fc-cli -- platform
```

`fc-cli start` notes:

- `--backend firecracker`:
  uses `--socket-path` (default `/tmp/firecracker.socket`).
- `--backend jailer`:
  requires `--uid` and `--gid`; custom `--socket-path` is not supported.
- `--backend jailer --daemonize`:
  must be used together with `--detach`.
- `--detach`:
  leaves the process running and prints `socket` plus best-effort `pid`.
- default (without `--detach`):
  keeps `fc-cli` attached; press `Ctrl+C` for graceful shutdown.

## Building

Requires Node.js (for `npx swagger2openapi` during code generation).

```bash
cargo build
```

## License

MIT OR Apache-2.0
