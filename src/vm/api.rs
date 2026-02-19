use crate::error::{VmError, VirtualGhostError};
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::{Request, Response, Uri};
use std::path::{Path, PathBuf};

use super::models::*;

pub struct FirecrackerClient {
    socket_path: PathBuf,
}

impl FirecrackerClient {
    pub fn new(socket_path: &Path) -> Self {
        Self {
            socket_path: socket_path.to_path_buf(),
        }
    }

    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    pub async fn set_machine_config(&self, config: &MachineConfig) -> Result<(), VirtualGhostError> {
        let body = serde_json::to_string(config).unwrap();
        self.put("/machine-config", &body).await
    }

    pub async fn set_boot_source(&self, source: &BootSource) -> Result<(), VirtualGhostError> {
        let body = serde_json::to_string(source).unwrap();
        self.put("/boot-source", &body).await
    }

    pub async fn set_drive(&self, drive: &Drive) -> Result<(), VirtualGhostError> {
        let body = serde_json::to_string(drive).unwrap();
        let path = format!("/drives/{}", drive.drive_id);
        self.put(&path, &body).await
    }

    pub async fn set_vsock(&self, vsock: &VsockConfig) -> Result<(), VirtualGhostError> {
        let body = serde_json::to_string(vsock).unwrap();
        self.put("/vsock", &body).await
    }

    pub async fn start_instance(&self) -> Result<(), VirtualGhostError> {
        let action = Action {
            action_type: ActionType::InstanceStart,
        };
        let body = serde_json::to_string(&action).unwrap();
        self.put("/actions", &body).await
    }

    async fn put(&self, path: &str, body: &str) -> Result<(), VirtualGhostError> {
        let response = self.send_request("PUT", path, Some(body)).await?;
        let status = response.status().as_u16();
        if status >= 400 {
            let body_bytes = response.into_body().collect().await
                .map_err(|e| VmError::ApiError {
                    status,
                    body: e.to_string(),
                })?;
            let body_str = String::from_utf8_lossy(&body_bytes.to_bytes()).to_string();
            return Err(VmError::ApiError { status, body: body_str }.into());
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
        // For now, build the request structure to validate the API design
        let _uri: Uri = format!("http://localhost{path}").parse().unwrap();
        let _request = Request::builder()
            .method(method)
            .uri(format!("http://localhost{path}"))
            .header("Content-Type", "application/json")
            .body(Full::new(Bytes::from(
                body.unwrap_or("").to_string(),
            )))
            .unwrap();

        // Placeholder: actual Unix socket connection will use hyperlocal
        let _ = &self.socket_path;
        todo!("Connect to Firecracker Unix socket at {:?}", self.socket_path)
    }
}
