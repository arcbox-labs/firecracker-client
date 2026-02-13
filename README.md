# Firecracker Rust Client SDK

Rust SDK for the [Firecracker](https://github.com/firecracker-microvm/firecracker) microVM API.

## Crates

| Crate | Description |
|---|---|
| `firecracker` | Facade — re-exports `api` + `sdk` |
| `fc-api` | Low-level typed client, generated from Swagger spec via [progenitor](https://github.com/oxidecomputer/progenitor) |
| `fc-sdk` | High-level typestate wrapper: `VmBuilder` → `Vm` lifecycle |

## Quick Start

```rust
use firecracker::sdk::{VmBuilder, types};

let vm = VmBuilder::new("/tmp/firecracker.socket")
    .boot_source(types::BootSource {
        kernel_image_path: "/path/to/vmlinux".into(),
        boot_args: Some("console=ttyS0".into()),
        initrd_path: None,
    })
    .machine_config(types::MachineConfiguration {
        vcpu_count: 2,
        mem_size_mib: 256,
        ..Default::default()
    })
    .start()
    .await?;

vm.pause().await?;
vm.resume().await?;
vm.snapshot("/tmp/snap", "/tmp/mem").await?;
```

For operations not covered by the SDK, use the generated API client directly:

```rust
use firecracker::api::Client;

let client = vm.client();
client.describe_instance().send().await?;
```

## Bundled Runtime Mode

`fc-sdk` supports resolving `firecracker` and `jailer` binaries from bundled
Firecracker upstream release artifacts with optional fallback to system `PATH`.
Release-based bundled mode currently supports:

- `linux-x86_64`
- `linux-aarch64`

```rust
use firecracker::sdk::{
    BundledMode, BundledRuntimeOptions, FirecrackerProcessBuilder, JailerProcessBuilder,
};

let bundled = BundledRuntimeOptions::new()
    .mode(BundledMode::BundledThenSystem)
    .bundle_root("/opt/arcbox/bundled")
    .release_version("v1.12.1")
    .firecracker_sha256("sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
    .jailer_sha256("sha256:abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789");

let fc = FirecrackerProcessBuilder::new_with_bundled_options(
    "/tmp/firecracker.socket",
    bundled.clone(),
)?;

let jailer = JailerProcessBuilder::new_with_bundled_options("vm-1", 1000, 1000, bundled)?;
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

## Building

Requires Node.js (for `npx swagger2openapi` during code generation).

```bash
cargo build
```

## License

MIT OR Apache-2.0
