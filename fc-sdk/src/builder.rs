use std::path::Path;

use fc_api::Client;
use fc_api::types::{
    Balloon, BootSource, CpuConfig, Drive, EntropyDevice, FullVmConfiguration, Logger,
    MachineConfiguration, MemoryHotplugConfig, Metrics, MmdsConfig, NetworkInterface, Pmem,
    SerialDevice, Vsock,
};

use crate::error::{Error, Result};
use crate::vm::Vm;

/// Pre-boot VM configuration builder.
///
/// Accumulates configuration and sends it to Firecracker upon [`start()`](Self::start).
///
/// # Required Configuration
///
/// - [`boot_source()`](Self::boot_source) — kernel image path (required)
/// - [`machine_config()`](Self::machine_config) — vCPU count and memory size (required)
///
/// # Example
///
/// ```no_run
/// use fc_sdk::{VmBuilder, types::*};
///
/// # async fn example() -> fc_sdk::Result<()> {
/// let vm = VmBuilder::new("/tmp/firecracker.sock")
///     .boot_source(BootSource {
///         kernel_image_path: "/path/to/vmlinux".into(),
///         boot_args: Some("console=ttyS0 reboot=k panic=1".into()),
///         initrd_path: None,
///     })
///     .machine_config(MachineConfiguration {
///         vcpu_count: std::num::NonZeroU64::new(2).unwrap(),
///         mem_size_mib: 512,
///         smt: false,
///         track_dirty_pages: false,
///         cpu_template: None,
///         huge_pages: None,
///     })
///     .drive(Drive {
///         drive_id: "rootfs".into(),
///         path_on_host: Some("/path/to/rootfs.ext4".into()),
///         is_root_device: true,
///         is_read_only: Some(false),
///         cache_type: DriveCacheType::Unsafe,
///         io_engine: DriveIoEngine::Sync,
///         partuuid: None,
///         rate_limiter: None,
///         socket: None,
///     })
///     .start()
///     .await?;
/// # Ok(())
/// # }
/// ```
pub struct VmBuilder {
    client: Client,
    boot_source: Option<BootSource>,
    machine_config: Option<MachineConfiguration>,
    cpu_config: Option<CpuConfig>,
    drives: Vec<Drive>,
    pmem_devices: Vec<Pmem>,
    network_interfaces: Vec<NetworkInterface>,
    balloon: Option<Balloon>,
    vsock: Option<Vsock>,
    entropy: Option<EntropyDevice>,
    serial: Option<SerialDevice>,
    memory_hotplug: Option<MemoryHotplugConfig>,
    mmds_config: Option<MmdsConfig>,
    mmds_data: Option<serde_json::Map<String, serde_json::Value>>,
    logger: Option<Logger>,
    metrics: Option<Metrics>,
}

impl VmBuilder {
    /// Create a new builder connected to the Firecracker socket at `socket_path`.
    pub fn new(socket_path: impl AsRef<Path>) -> Self {
        let client = crate::connection::connect(socket_path);
        Self {
            client,
            boot_source: None,
            machine_config: None,
            cpu_config: None,
            drives: Vec::new(),
            pmem_devices: Vec::new(),
            network_interfaces: Vec::new(),
            balloon: None,
            vsock: None,
            entropy: None,
            serial: None,
            memory_hotplug: None,
            mmds_config: None,
            mmds_data: None,
            logger: None,
            metrics: None,
        }
    }

    /// Create a new builder using an existing API client.
    pub fn with_client(client: Client) -> Self {
        Self {
            client,
            boot_source: None,
            machine_config: None,
            cpu_config: None,
            drives: Vec::new(),
            pmem_devices: Vec::new(),
            network_interfaces: Vec::new(),
            balloon: None,
            vsock: None,
            entropy: None,
            serial: None,
            memory_hotplug: None,
            mmds_config: None,
            mmds_data: None,
            logger: None,
            metrics: None,
        }
    }

    /// Create a new builder pre-populated from a [`FullVmConfiguration`].
    ///
    /// This is useful for cloning or modifying an existing VM's configuration.
    pub fn from_config(socket_path: impl AsRef<Path>, config: FullVmConfiguration) -> Self {
        let client = crate::connection::connect(socket_path);
        Self::from_config_with_client(client, config)
    }

    /// Create a new builder pre-populated from a [`FullVmConfiguration`] using an existing client.
    pub fn from_config_with_client(client: Client, config: FullVmConfiguration) -> Self {
        Self {
            client,
            boot_source: config.boot_source,
            machine_config: config.machine_config,
            cpu_config: config.cpu_config,
            drives: config.drives,
            pmem_devices: config.pmem,
            network_interfaces: config.network_interfaces,
            balloon: config.balloon,
            vsock: config.vsock,
            entropy: config.entropy,
            serial: None, // Not available in FullVmConfiguration
            memory_hotplug: config.memory_hotplug,
            mmds_config: config.mmds_config,
            mmds_data: None,
            logger: config.logger,
            metrics: config.metrics,
        }
    }

    // =========================================================================
    // Required Configuration
    // =========================================================================

    /// Set the boot source (kernel image path and optional boot arguments).
    ///
    /// **Required** — the VM cannot start without a boot source.
    pub fn boot_source(mut self, boot_source: BootSource) -> Self {
        self.boot_source = Some(boot_source);
        self
    }

    /// Set the machine configuration (vCPU count, memory size, etc.).
    ///
    /// **Required** — the VM cannot start without machine configuration.
    pub fn machine_config(mut self, machine_config: MachineConfiguration) -> Self {
        self.machine_config = Some(machine_config);
        self
    }

    // =========================================================================
    // Optional Configuration
    // =========================================================================

    /// Set CPU configuration (CPUID/MSR modifiers on x86_64, register modifiers on aarch64).
    pub fn cpu_config(mut self, cpu_config: CpuConfig) -> Self {
        self.cpu_config = Some(cpu_config);
        self
    }

    /// Add a block device (drive).
    pub fn drive(mut self, drive: Drive) -> Self {
        self.drives.push(drive);
        self
    }

    /// Add a root drive (convenience method that sets `is_root_device` to true).
    pub fn root_drive(mut self, mut drive: Drive) -> Self {
        drive.is_root_device = true;
        self.drives.push(drive);
        self
    }

    /// Add a virtio-pmem persistent memory device.
    pub fn pmem(mut self, pmem: Pmem) -> Self {
        self.pmem_devices.push(pmem);
        self
    }

    /// Add a network interface.
    pub fn network_interface(mut self, iface: NetworkInterface) -> Self {
        self.network_interfaces.push(iface);
        self
    }

    /// Configure the balloon device for memory ballooning.
    pub fn balloon(mut self, balloon: Balloon) -> Self {
        self.balloon = Some(balloon);
        self
    }

    /// Configure a vsock device for host-guest communication.
    pub fn vsock(mut self, vsock: Vsock) -> Self {
        self.vsock = Some(vsock);
        self
    }

    /// Configure an entropy device for high-quality random data.
    pub fn entropy(mut self, entropy: EntropyDevice) -> Self {
        self.entropy = Some(entropy);
        self
    }

    /// Configure serial console output redirection.
    pub fn serial(mut self, serial: SerialDevice) -> Self {
        self.serial = Some(serial);
        self
    }

    /// Configure virtio-mem hotpluggable memory.
    pub fn memory_hotplug(mut self, memory_hotplug: MemoryHotplugConfig) -> Self {
        self.memory_hotplug = Some(memory_hotplug);
        self
    }

    /// Configure MMDS (Microvm Metadata Service).
    pub fn mmds_config(mut self, mmds_config: MmdsConfig) -> Self {
        self.mmds_config = Some(mmds_config);
        self
    }

    /// Set the initial MMDS data store contents.
    ///
    /// The MMDS config must also be set via [`mmds_config()`](Self::mmds_config) for this
    /// to take effect. The data is applied after the MMDS config during [`start()`](Self::start).
    pub fn mmds_data(mut self, data: serde_json::Map<String, serde_json::Value>) -> Self {
        self.mmds_data = Some(data);
        self
    }

    /// Configure logging output.
    pub fn logger(mut self, logger: Logger) -> Self {
        self.logger = Some(logger);
        self
    }

    /// Configure metrics output.
    pub fn metrics(mut self, metrics: Metrics) -> Self {
        self.metrics = Some(metrics);
        self
    }

    // =========================================================================
    // Build and Start
    // =========================================================================

    /// Apply all configuration and start the microVM.
    ///
    /// Returns a [`Vm`] handle for post-boot operations.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `boot_source` is not configured
    /// - `machine_config` is not configured
    /// - Any API call fails
    pub async fn start(self) -> Result<Vm> {
        let boot_source = self
            .boot_source
            .ok_or(Error::MissingConfig("boot_source"))?;
        let machine_config = self
            .machine_config
            .ok_or(Error::MissingConfig("machine_config"))?;

        // Apply logger first (if configured) — must be done before other config
        if let Some(logger) = self.logger {
            self.client.put_logger().body(logger).send().await?;
        }

        // Apply metrics (if configured) — must be done before other config
        if let Some(metrics) = self.metrics {
            self.client.put_metrics().body(metrics).send().await?;
        }

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

        // Apply CPU configuration (if configured)
        if let Some(cpu_config) = self.cpu_config {
            self.client
                .put_cpu_configuration()
                .body(cpu_config)
                .send()
                .await?;
        }

        // Apply drives
        for drive in &self.drives {
            self.client
                .put_guest_drive_by_id()
                .drive_id(&drive.drive_id)
                .body(drive.clone())
                .send()
                .await?;
        }

        // Apply pmem devices
        for pmem in &self.pmem_devices {
            self.client
                .put_guest_pmem_by_id()
                .id(&pmem.id)
                .body(pmem.clone())
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

        // Apply balloon (if configured)
        if let Some(balloon) = self.balloon {
            self.client.put_balloon().body(balloon).send().await?;
        }

        // Apply vsock (if configured)
        if let Some(vsock) = self.vsock {
            self.client.put_guest_vsock().body(vsock).send().await?;
        }

        // Apply entropy device (if configured)
        if let Some(entropy) = self.entropy {
            self.client
                .put_entropy_device()
                .body(entropy)
                .send()
                .await?;
        }

        // Apply serial device (if configured)
        if let Some(serial) = self.serial {
            self.client.put_serial_device().body(serial).send().await?;
        }

        // Apply memory hotplug (if configured)
        if let Some(memory_hotplug) = self.memory_hotplug {
            self.client
                .put_memory_hotplug()
                .body(memory_hotplug)
                .send()
                .await?;
        }

        // Apply MMDS config (if configured)
        if let Some(mmds_config) = self.mmds_config {
            self.client
                .put_mmds_config()
                .body(mmds_config)
                .send()
                .await?;
        }

        // Apply MMDS data (if configured)
        if let Some(mmds_data) = self.mmds_data {
            self.client.put_mmds().body(mmds_data).send().await?;
        }

        // Start the instance
        self.client
            .create_sync_action()
            .body_map(|b| b.action_type(fc_api::types::InstanceActionInfoActionType::InstanceStart))
            .send()
            .await?;

        Ok(Vm::new(self.client))
    }

    /// Get a reference to the underlying API client.
    pub fn client(&self) -> &Client {
        &self.client
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;

    use super::*;

    #[test]
    fn test_from_config_maps_all_fields() {
        let config = FullVmConfiguration {
            boot_source: Some(BootSource {
                kernel_image_path: "/path/to/kernel".into(),
                boot_args: Some("console=ttyS0".into()),
                initrd_path: None,
            }),
            machine_config: Some(MachineConfiguration {
                vcpu_count: NonZeroU64::new(4).unwrap(),
                mem_size_mib: 1024,
                smt: true,
                track_dirty_pages: true,
                cpu_template: None,
                huge_pages: None,
            }),
            cpu_config: None,
            drives: vec![Drive {
                drive_id: "rootfs".into(),
                path_on_host: Some("/path/to/rootfs.ext4".into()),
                is_root_device: true,
                is_read_only: Some(false),
                partuuid: None,
                cache_type: fc_api::types::DriveCacheType::Unsafe,
                rate_limiter: None,
                io_engine: fc_api::types::DriveIoEngine::Sync,
                socket: None,
            }],
            pmem: vec![],
            network_interfaces: vec![NetworkInterface {
                iface_id: "eth0".into(),
                guest_mac: Some("AA:BB:CC:DD:EE:FF".into()),
                host_dev_name: "tap0".into(),
                rx_rate_limiter: None,
                tx_rate_limiter: None,
            }],
            balloon: None,
            vsock: None,
            entropy: None,
            memory_hotplug: None,
            mmds_config: None,
            logger: None,
            metrics: None,
        };

        let builder = VmBuilder::from_config("/tmp/test.sock", config);

        // Verify required fields are mapped
        assert!(builder.boot_source.is_some());
        assert_eq!(
            builder.boot_source.as_ref().unwrap().kernel_image_path,
            "/path/to/kernel"
        );
        assert!(builder.machine_config.is_some());
        assert_eq!(
            builder.machine_config.as_ref().unwrap().vcpu_count,
            NonZeroU64::new(4).unwrap()
        );
        assert_eq!(builder.machine_config.as_ref().unwrap().mem_size_mib, 1024);

        // Verify collection fields
        assert_eq!(builder.drives.len(), 1);
        assert_eq!(builder.drives[0].drive_id, "rootfs");
        assert_eq!(builder.pmem_devices.len(), 0);
        assert_eq!(builder.network_interfaces.len(), 1);
        assert_eq!(builder.network_interfaces[0].iface_id, "eth0");

        // Verify optional fields default to None
        assert!(builder.cpu_config.is_none());
        assert!(builder.balloon.is_none());
        assert!(builder.vsock.is_none());
        assert!(builder.entropy.is_none());
        assert!(builder.serial.is_none()); // Not in FullVmConfiguration
        assert!(builder.memory_hotplug.is_none());
        assert!(builder.mmds_config.is_none());
        assert!(builder.mmds_data.is_none());
        assert!(builder.logger.is_none());
        assert!(builder.metrics.is_none());
    }
}
