use std::path::Path;

use firecracker_api::types::{BootSource, Drive, MachineConfiguration, NetworkInterface};
use firecracker_api::Client;

use crate::error::{Error, Result};
use crate::vm::Vm;

/// Pre-boot VM configuration builder.
///
/// Accumulates configuration and sends it to Firecracker upon [`start()`](Self::start).
pub struct VmBuilder {
    client: Client,
    boot_source: Option<BootSource>,
    machine_config: Option<MachineConfiguration>,
    drives: Vec<Drive>,
    network_interfaces: Vec<NetworkInterface>,
}

impl VmBuilder {
    /// Create a new builder connected to the Firecracker socket at `socket_path`.
    pub fn new(socket_path: impl AsRef<Path>) -> Self {
        let client = crate::connection::connect(socket_path);
        Self {
            client,
            boot_source: None,
            machine_config: None,
            drives: Vec::new(),
            network_interfaces: Vec::new(),
        }
    }

    /// Set the boot source (kernel image path and optional boot arguments).
    pub fn boot_source(mut self, boot_source: BootSource) -> Self {
        self.boot_source = Some(boot_source);
        self
    }

    /// Set the machine configuration (vCPU count, memory size, etc.).
    pub fn machine_config(mut self, machine_config: MachineConfiguration) -> Self {
        self.machine_config = Some(machine_config);
        self
    }

    /// Add a block device (drive).
    pub fn drive(mut self, drive: Drive) -> Self {
        self.drives.push(drive);
        self
    }

    /// Add a network interface.
    pub fn network_interface(mut self, iface: NetworkInterface) -> Self {
        self.network_interfaces.push(iface);
        self
    }

    /// Apply all configuration and start the microVM.
    ///
    /// Returns a [`Vm`] handle for post-boot operations.
    pub async fn start(self) -> Result<Vm> {
        let boot_source = self
            .boot_source
            .ok_or(Error::MissingConfig("boot_source"))?;
        let machine_config = self
            .machine_config
            .ok_or(Error::MissingConfig("machine_config"))?;

        // Apply boot source
        self.client
            .put_guest_boot_source()
            .body(boot_source)
            .send()
            .await?;

        // Apply machine configuration
        self.client
            .put_machine_configuration()
            .body(machine_config)
            .send()
            .await?;

        // Apply drives
        for drive in &self.drives {
            self.client
                .put_guest_drive_by_id()
                .drive_id(&drive.drive_id)
                .body(drive.clone())
                .send()
                .await?;
        }

        // Apply network interfaces
        for iface in &self.network_interfaces {
            self.client
                .put_guest_network_interface_by_id()
                .iface_id(&iface.iface_id)
                .body(iface.clone())
                .send()
                .await?;
        }

        // Start the instance
        self.client
            .create_sync_action()
            .body_map(|b| b.action_type(firecracker_api::types::InstanceActionInfoActionType::InstanceStart))
            .send()
            .await?;

        Ok(Vm::new(self.client))
    }
}
