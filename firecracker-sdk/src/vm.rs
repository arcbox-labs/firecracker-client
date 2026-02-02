use firecracker_api::types::{
    InstanceActionInfoActionType, SnapshotCreateParamsSnapshotType, VmState,
};
use firecracker_api::Client;

use crate::error::Result;

/// Handle to a running Firecracker microVM.
///
/// Obtained from [`VmBuilder::start()`](crate::VmBuilder::start).
pub struct Vm {
    client: Client,
}

impl Vm {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
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

    /// Flush metrics.
    pub async fn flush_metrics(&self) -> Result<()> {
        self.client
            .create_sync_action()
            .body_map(|b| b.action_type(InstanceActionInfoActionType::FlushMetrics))
            .send()
            .await?;
        Ok(())
    }

    /// Create a snapshot of the running VM.
    pub async fn snapshot(&self, snapshot_path: &str, mem_file_path: &str) -> Result<()> {
        self.client
            .create_snapshot()
            .body_map(|b| {
                b.mem_file_path(mem_file_path.to_owned())
                    .snapshot_path(snapshot_path.to_owned())
                    .snapshot_type(SnapshotCreateParamsSnapshotType::Full)
            })
            .send()
            .await?;
        Ok(())
    }

    /// Get a reference to the underlying API client.
    pub fn client(&self) -> &Client {
        &self.client
    }
}
