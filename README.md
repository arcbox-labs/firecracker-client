# Firecracker Rust Client SDK

Rust SDK for the [Firecracker](https://github.com/firecracker-microvm/firecracker) microVM API.

## Crates

| Crate | Description |
|---|---|
| `firecracker` | Facade — re-exports `api` + `sdk` |
| `firecracker-api` | Low-level typed client, generated from Swagger spec via [progenitor](https://github.com/oxidecomputer/progenitor) |
| `firecracker-sdk` | High-level typestate wrapper: `VmBuilder` → `Vm` lifecycle |

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

## Building

Requires Node.js (for `npx swagger2openapi` during code generation).

```bash
cargo build
```

## License

MIT OR Apache-2.0
