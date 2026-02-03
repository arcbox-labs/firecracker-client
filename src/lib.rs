//! Rust SDK for the Firecracker microVM API.
//!
//! This crate provides two modules:
//!
//! - [`api`] — Low-level typed API client generated via progenitor
//! - [`sdk`] — High-level typestate SDK for managing VM lifecycles
//!
//! # Quick Start
//!
//! ```no_run
//! use firecracker::sdk::{VmBuilder, types::*};
//!
//! # async fn example() -> firecracker::sdk::Result<()> {
//! let vm = VmBuilder::new("/tmp/firecracker.sock")
//!     .boot_source(BootSource {
//!         kernel_image_path: "/path/to/vmlinux".into(),
//!         boot_args: Some("console=ttyS0 reboot=k panic=1".into()),
//!         initrd_path: None,
//!     })
//!     .machine_config(MachineConfiguration {
//!         vcpu_count: 2,
//!         mem_size_mib: 512,
//!         smt: None,
//!         track_dirty_pages: None,
//!         cpu_template: None,
//!         huge_pages: None,
//!     })
//!     .start()
//!     .await?;
//!
//! // Post-boot operations
//! let info = vm.describe().await?;
//! println!("VM state: {:?}", info.state);
//! # Ok(())
//! # }
//! ```

/// Low-level typed API client generated via progenitor.
///
/// This module exposes the raw Firecracker REST API as Rust types and async methods.
/// Use this for fine-grained control over API calls or for operations not covered
/// by the high-level SDK.
pub use fc_api as api;

/// High-level typestate SDK for managing Firecracker microVM lifecycles.
///
/// This module provides:
/// - [`sdk::VmBuilder`] — Pre-boot configuration builder
/// - [`sdk::Vm`] — Post-boot VM operations
/// - [`sdk::restore`] — Restore from snapshot
/// - [`sdk::types`] — Re-exported API types
pub use fc_sdk as sdk;
