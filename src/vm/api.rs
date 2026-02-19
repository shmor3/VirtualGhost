#![cfg(unix)]

use crate::error::{VmError, VirtualGhostError};
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::{Request, Response};
use std::path::{Path, PathBuf};

use super::models::*;

pub struct CloudHypervisorClient {
    socket_path: PathBuf,
}

impl CloudHypervisorClient {
    pub fn new(socket_path: &Path) -> Self {
        Self {
            socket_path: socket_path.to_path_buf(),
        }
    }

    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    /// Create a VM with the given configuration (PUT /api/v1/vm.create)
    pub async fn vm_create(&self, config: &VmCreateConfig) -> Result<(), VirtualGhostError> {
        let body = serde_json::to_string(config).unwrap();
        self.put("/api/v1/vm.create", Some(&body)).await
    }

    /// Boot the VM (PUT /api/v1/vm.boot)
    pub async fn vm_boot(&self) -> Result<(), VirtualGhostError> {
        self.put("/api/v1/vm.boot", None).await
    }

    /// Shutdown the VM gracefully (PUT /api/v1/vm.shutdown)
    pub async fn vm_shutdown(&self) -> Result<(), VirtualGhostError> {
        self.put("/api/v1/vm.shutdown", None).await
    }

    /// Delete the VM (PUT /api/v1/vm.delete)
    pub async fn vm_delete(&self) -> Result<(), VirtualGhostError> {
        self.put("/api/v1/vm.delete", None).await
    }

    /// Shutdown the VMM process (PUT /api/v1/vmm.shutdown)
    pub async fn vmm_shutdown(&self) -> Result<(), VirtualGhostError> {
        self.put("/api/v1/vmm.shutdown", None).await
    }

    async fn put(&self, path: &str, body: Option<&str>) -> Result<(), VirtualGhostError> {
        let response = self.send_request("PUT", path, body).await?;
        let status = response.status().as_u16();
        if status >= 400 {
            let body_bytes = response
                .into_body()
                .collect()
                .await
                .map_err(|e| VmError::ApiError {
                    status,
                    body: e.to_string(),
                })?;
            let body_str = String::from_utf8_lossy(&body_bytes.to_bytes()).to_string();
            return Err(VmError::ApiError {
                status,
                body: body_str,
            }
            .into());
        }
        Ok(())
    }

    async fn send_request(
        &self,
        method: &str,
        path: &str,
        body: Option<&str>,
    ) -> Result<Response<Incoming>, VirtualGhostError> {
        // TODO: Implement HTTP-over-Unix-socket using hyperlocal
        let _request = Request::builder()
            .method(method)
            .uri(format!("http://localhost{path}"))
            .header("Content-Type", "application/json")
            .body(Full::new(Bytes::from(
                body.unwrap_or("").to_string(),
            )))
            .unwrap();

        let _ = &self.socket_path;
        todo!(
            "Connect to Cloud Hypervisor Unix socket at {:?}",
            self.socket_path
        )
    }
}
