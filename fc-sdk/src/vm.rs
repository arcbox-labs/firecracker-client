use std::path::Path;

use fc_api::types::{
    Balloon, BalloonHintingStatus, BalloonStartCmd, BalloonStats, BalloonStatsUpdate,
    BalloonUpdate, FirecrackerVersion, FullVmConfiguration, InstanceActionInfoActionType,
    InstanceInfo, MachineConfiguration, MemoryHotplugSizeUpdate, MemoryHotplugStatus,
    PartialDrive, PartialNetworkInterface, SnapshotCreateParams, SnapshotCreateParamsSnapshotType,
    SnapshotLoadParams, VmState,
};
use fc_api::Client;

use crate::connection::connect;
use crate::error::Result;

/// Handle to a running Firecracker microVM.
///
/// Obtained from [`VmBuilder::start()`](crate::VmBuilder::start) or [`restore()`].
pub struct Vm {
    client: Client,
}

impl Vm {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    // =========================================================================
    // Instance Management
    // =========================================================================

    /// Get general information about the instance.
    pub async fn describe(&self) -> Result<InstanceInfo> {
        let info = self.client.describe_instance().send().await?;
        Ok(info.into_inner())
    }

    /// Get the Firecracker version.
    pub async fn version(&self) -> Result<FirecrackerVersion> {
        let version = self.client.get_firecracker_version().send().await?;
        Ok(version.into_inner())
    }

    /// Get the full VM configuration.
    pub async fn config(&self) -> Result<FullVmConfiguration> {
        let config = self.client.get_export_vm_config().send().await?;
        Ok(config.into_inner())
    }

    /// Pause the microVM.
    pub async fn pause(&self) -> Result<()> {
        self.client
            .patch_vm()
            .body_map(|b| b.state(VmState::Paused))
            .send()
            .await?;
        Ok(())
    }

    /// Resume a paused microVM.
    pub async fn resume(&self) -> Result<()> {
        self.client
            .patch_vm()
            .body_map(|b| b.state(VmState::Resumed))
            .send()
            .await?;
        Ok(())
    }

    /// Send Ctrl+Alt+Del to the guest.
    pub async fn send_ctrl_alt_del(&self) -> Result<()> {
        self.client
            .create_sync_action()
            .body_map(|b| b.action_type(InstanceActionInfoActionType::SendCtrlAltDel))
            .send()
            .await?;
        Ok(())
    }

    /// Flush metrics to the configured metrics path.
    pub async fn flush_metrics(&self) -> Result<()> {
        self.client
            .create_sync_action()
            .body_map(|b| b.action_type(InstanceActionInfoActionType::FlushMetrics))
            .send()
            .await?;
        Ok(())
    }

    // =========================================================================
    // Snapshots
    // =========================================================================

    /// Create a full snapshot of the microVM.
    ///
    /// The VM should be paused before creating a snapshot.
    pub async fn create_snapshot(&self, snapshot_path: &str, mem_file_path: &str) -> Result<()> {
        self.client
            .create_snapshot()
            .body(SnapshotCreateParams {
                mem_file_path: mem_file_path.to_owned(),
                snapshot_path: snapshot_path.to_owned(),
                snapshot_type: Some(SnapshotCreateParamsSnapshotType::Full),
            })
            .send()
            .await?;
        Ok(())
    }

    /// Create a diff snapshot of the microVM.
    ///
    /// The VM should be paused before creating a snapshot.
    /// Requires `track_dirty_pages` to be enabled in machine configuration.
    pub async fn create_diff_snapshot(
        &self,
        snapshot_path: &str,
        mem_file_path: &str,
    ) -> Result<()> {
        self.client
            .create_snapshot()
            .body(SnapshotCreateParams {
                mem_file_path: mem_file_path.to_owned(),
                snapshot_path: snapshot_path.to_owned(),
                snapshot_type: Some(SnapshotCreateParamsSnapshotType::Diff),
            })
            .send()
            .await?;
        Ok(())
    }

    // =========================================================================
    // Live Updates - Drives
    // =========================================================================

    /// Update a drive's properties (hot swap or rate limiting).
    pub async fn update_drive(&self, drive_id: &str, update: PartialDrive) -> Result<()> {
        self.client
            .patch_guest_drive_by_id()
            .drive_id(drive_id)
            .body(update)
            .send()
            .await?;
        Ok(())
    }

    // =========================================================================
    // Live Updates - Network
    // =========================================================================

    /// Update a network interface's rate limiters.
    pub async fn update_network_interface(
        &self,
        iface_id: &str,
        update: PartialNetworkInterface,
    ) -> Result<()> {
        self.client
            .patch_guest_network_interface_by_id()
            .iface_id(iface_id)
            .body(update)
            .send()
            .await?;
        Ok(())
    }

    // =========================================================================
    // Live Updates - Balloon
    // =========================================================================

    /// Get the current balloon device configuration.
    pub async fn balloon_config(&self) -> Result<Balloon> {
        let balloon = self.client.describe_balloon_config().send().await?;
        Ok(balloon.into_inner())
    }

    /// Get balloon device statistics.
    pub async fn balloon_stats(&self) -> Result<BalloonStats> {
        let stats = self.client.describe_balloon_stats().send().await?;
        Ok(stats.into_inner())
    }

    /// Update the balloon device target size.
    pub async fn update_balloon(&self, amount_mib: i64) -> Result<()> {
        self.client
            .patch_balloon()
            .body(BalloonUpdate { amount_mib })
            .send()
            .await?;
        Ok(())
    }

    /// Update the balloon statistics polling interval.
    pub async fn update_balloon_stats_interval(&self, stats_polling_interval_s: i64) -> Result<()> {
        self.client
            .patch_balloon_stats_interval()
            .body(BalloonStatsUpdate {
                stats_polling_interval_s,
            })
            .send()
            .await?;
        Ok(())
    }

    // =========================================================================
    // Balloon Hinting
    // =========================================================================

    /// Start a free page hinting run.
    pub async fn start_balloon_hinting(&self, acknowledge_on_stop: Option<bool>) -> Result<()> {
        self.client
            .start_balloon_hinting()
            .body(BalloonStartCmd {
                acknowledge_on_stop,
            })
            .send()
            .await?;
        Ok(())
    }

    /// Get the balloon hinting status.
    pub async fn balloon_hinting_status(&self) -> Result<BalloonHintingStatus> {
        let status = self.client.describe_balloon_hinting().send().await?;
        Ok(status.into_inner())
    }

    /// Stop a free page hinting run.
    pub async fn stop_balloon_hinting(&self) -> Result<()> {
        self.client.stop_balloon_hinting().send().await?;
        Ok(())
    }

    // =========================================================================
    // Live Updates - Memory Hotplug
    // =========================================================================

    /// Get the current machine configuration.
    pub async fn machine_configuration(&self) -> Result<MachineConfiguration> {
        let config = self.client.get_machine_configuration().send().await?;
        Ok(config.into_inner())
    }

    /// Partially update the machine configuration.
    ///
    /// Pre-boot only. If any parameter has an incorrect value, the whole update fails.
    pub async fn update_machine_config(&self, config: MachineConfiguration) -> Result<()> {
        self.client
            .patch_machine_configuration()
            .body(config)
            .send()
            .await?;
        Ok(())
    }

    /// Get the status of the hotpluggable memory device.
    pub async fn memory_hotplug_status(&self) -> Result<MemoryHotplugStatus> {
        let status = self.client.get_memory_hotplug().send().await?;
        Ok(status.into_inner())
    }

    /// Update the size of the hotpluggable memory region.
    pub async fn update_memory_hotplug(&self, requested_size_mib: Option<i64>) -> Result<()> {
        self.client
            .patch_memory_hotplug()
            .body(MemoryHotplugSizeUpdate { requested_size_mib })
            .send()
            .await?;
        Ok(())
    }

    // =========================================================================
    // MMDS (Microvm Metadata Service)
    // =========================================================================

    /// Get the MMDS data store contents.
    pub async fn get_mmds(&self) -> Result<serde_json::Map<String, serde_json::Value>> {
        let mmds = self.client.get_mmds().send().await?;
        Ok(mmds.into_inner())
    }

    /// Set (replace) the MMDS data store contents.
    pub async fn set_mmds(&self, data: serde_json::Map<String, serde_json::Value>) -> Result<()> {
        self.client.put_mmds().body(data).send().await?;
        Ok(())
    }

    /// Patch (merge) the MMDS data store contents.
    pub async fn patch_mmds(&self, data: serde_json::Map<String, serde_json::Value>) -> Result<()> {
        self.client.patch_mmds().body(data).send().await?;
        Ok(())
    }

    // =========================================================================
    // Direct Client Access
    // =========================================================================

    /// Get a reference to the underlying API client for advanced operations.
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Consume the Vm and return the underlying API client.
    pub fn into_client(self) -> Client {
        self.client
    }
}

// =============================================================================
// Standalone Functions
// =============================================================================

/// Restore a microVM from a snapshot.
///
/// This must be called on a fresh Firecracker process (before configuring any
/// resources other than logger and metrics).
///
/// # Arguments
///
/// * `socket_path` - Path to the Firecracker Unix socket
/// * `params` - Snapshot load parameters
///
/// # Example
///
/// ```no_run
/// use fc_sdk::{restore, types::*};
///
/// # async fn example() -> fc_sdk::Result<()> {
/// let vm = restore(
///     "/tmp/firecracker.sock",
///     SnapshotLoadParams {
///         snapshot_path: "/path/to/snapshot".to_owned(),
///         mem_file_path: Some("/path/to/mem".to_owned()),
///         mem_backend: None,
///         enable_diff_snapshots: None,
///         track_dirty_pages: None,
///         resume_vm: Some(true),
///         network_overrides: None,
///     },
/// ).await?;
/// # Ok(())
/// # }
/// ```
pub async fn restore(socket_path: impl AsRef<Path>, params: SnapshotLoadParams) -> Result<Vm> {
    let client = connect(socket_path);
    client.load_snapshot().body(params).send().await?;
    Ok(Vm::new(client))
}

/// Restore a microVM from a snapshot using an existing client.
pub async fn restore_with_client(client: Client, params: SnapshotLoadParams) -> Result<Vm> {
    client.load_snapshot().body(params).send().await?;
    Ok(Vm::new(client))
}
