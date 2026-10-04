//! Pinned multilingual model downloads. No arbitrary URLs or renderer-supplied paths.
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager};
use tokio_util::sync::CancellationToken;

const REVISION: &str = "5359861c739e955e79d9a303bcbc70fb988958b1";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelId {
    Tiny,
    #[default]
    Base,
    Small,
}

impl ModelId {
    pub const ALL: [Self; 3] = [Self::Tiny, Self::Base, Self::Small];

    pub fn file(self) -> &'static str {
        match self {
            Self::Tiny => "ggml-tiny-q5_1.bin",
            Self::Base => "ggml-base-q5_1.bin",
            Self::Small => "ggml-small-q5_1.bin",
        }
    }

    pub fn bytes(self) -> u64 {
        match self {
            Self::Tiny => 32_152_673,
            Self::Base => 59_707_625,
            Self::Small => 190_085_487,
        }
    }

    fn sha256(self) -> &'static str {
        match self {
            Self::Tiny => "818710568da3ca15689e31a743197b520007872ff9576237bda97bd1b469c3d7",
            Self::Base => "422f1ae452ade6f30a004d7e5c6a43195e4433bc370bf23fac9cc591f01a8898",
            Self::Small => "ae85e4a935d7a567bd102fe55afc16bb595bdb618e11b2fc7591bc08120411bb",
        }
    }
}

#[derive(Clone, Default)]
pub struct ModelManager(Arc<Mutex<ModelState>>);

#[derive(Default)]
struct ModelState {
    download: Option<(ModelId, CancellationToken, u64)>,
    leases: Vec<ModelId>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    id: ModelId,
    bytes: u64,
    installed: bool,
    downloading: bool,
    downloaded_bytes: u64,
    in_use: bool,
}

pub fn directory(app: &AppHandle) -> Result<PathBuf> {
    let dir = app.path().app_local_data_dir()?.join("whisper-models");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

impl ModelManager {
    pub fn list(&self, dir: &Path) -> Vec<ModelInfo> {
        let state = self.0.lock().unwrap_or_else(|e| e.into_inner());
        ModelId::ALL
            .into_iter()
            .map(|id| {
                let download = state.download.as_ref().filter(|d| d.0 == id);
                ModelInfo {
                    id,
                    bytes: id.bytes(),
                    installed: fs::metadata(dir.join(id.file()))
                        .is_ok_and(|m| m.len() == id.bytes()),
                    downloading: download.is_some(),
                    downloaded_bytes: download.map_or(0, |d| d.2),
                    in_use: state.leases.contains(&id),
                }
            })
            .collect()
    }

    pub fn cancel_download(&self) {
        if let Some((_, token, _)) = &self.0.lock().unwrap_or_else(|e| e.into_inner()).download {
            token.cancel();
        }
    }

    pub fn remove(&self, dir: &Path, id: ModelId) -> Result<()> {
        let state = self.0.lock().unwrap_or_else(|e| e.into_inner());
        ensure!(
            !state.leases.contains(&id),
            "The model is being used by a session"
        );
        ensure!(state.download.is_none(), "A model download is in progress");
        let path = dir.join(id.file());
        match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    /// Pins the file against deletion or replacement for the lifetime of the context.
    pub fn lease(&self, dir: &Path, id: ModelId) -> Result<ModelLease> {
        let mut state = self.0.lock().unwrap_or_else(|e| e.into_inner());
        ensure!(
            state.download.as_ref().is_none_or(|d| d.0 != id),
            "The model is still downloading"
        );
        let path = dir.join(id.file());
        ensure!(
            path.is_file(),
            "Download the selected Whisper model before starting"
        );
        state.leases.push(id);
        Ok(ModelLease {
            manager: self.clone(),
            id,
            path,
        })
    }

    pub async fn download(&self, dir: &Path, id: ModelId) -> Result<()> {
        let token = CancellationToken::new();
        {
            let mut state = self.0.lock().unwrap_or_else(|e| e.into_inner());
            ensure!(
                state.download.is_none(),
                "Another model download is in progress"
            );
            ensure!(
                !state.leases.contains(&id),
                "The model is being used by a session"
            );
            state.download = Some((id, token.clone(), 0));
        }
        // RAII clears both the partial file and the operation even on error/unwind.
        let _operation = DownloadGuard(self.clone());
        let mut temp = tempfile::NamedTempFile::new_in(dir)?;
        let url = format!(
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/{REVISION}/{}",
            id.file()
        );
        let client = reqwest::Client::builder()
            .https_only(true)
            .connect_timeout(Duration::from_secs(15))
            .timeout(Duration::from_secs(1800))
            .build()?;
        let mut response = tokio::select! {
            _ = token.cancelled() => anyhow::bail!("Model download cancelled"),
            response = client.get(url).send() => response?.error_for_status()?,
        };
        let mut hash = Sha256::new();
        let mut total = 0u64;
        loop {
            // Cancellation also interrupts a stalled connection, not only a successful read.
            let chunk = tokio::select! {
                _ = token.cancelled() => anyhow::bail!("Model download cancelled"),
                chunk = tokio::time::timeout(Duration::from_secs(60), response.chunk()) =>
                    chunk.context("The model download stopped responding")??,
            };
            let Some(chunk) = chunk else {
                break;
            };
            total += chunk.len() as u64;
            ensure!(total <= id.bytes(), "The model exceeds its expected size");
            hash.update(&chunk);
            temp.write_all(&chunk)?;
            if let Some(download) = &mut self.0.lock().unwrap_or_else(|e| e.into_inner()).download {
                download.2 = total;
            }
        }
        ensure!(!token.is_cancelled(), "Model download cancelled");
        verify_digest(id, total, &format!("{:x}", hash.finalize()))?;
        temp.as_file().sync_all()?;
        temp.persist(dir.join(id.file()))
            .context("Could not install the verified model")?;
        Ok(())
    }
}

struct DownloadGuard(ModelManager);
impl Drop for DownloadGuard {
    fn drop(&mut self) {
        self.0 .0.lock().unwrap_or_else(|e| e.into_inner()).download = None;
    }
}

pub struct ModelLease {
    manager: ModelManager,
    id: ModelId,
    pub path: PathBuf,
}
impl ModelLease {
    /// Check cached bytes too: a corrupt/replaced local file must never reach native inference.
    pub fn read_verified(&self) -> Result<Vec<u8>> {
        let bytes = fs::metadata(&self.path)?.len();
        ensure!(
            bytes == self.id.bytes(),
            "Model size is wrong; remove and download it again"
        );
        let data = fs::read(&self.path)?;
        verify_digest(
            self.id,
            data.len() as u64,
            &format!("{:x}", Sha256::digest(&data)),
        )?;
        Ok(data)
    }
}
impl Drop for ModelLease {
    fn drop(&mut self) {
        let mut state = self.manager.0.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(i) = state.leases.iter().position(|id| *id == self.id) {
            state.leases.remove(i);
        }
    }
}

fn verify_digest(id: ModelId, bytes: u64, digest: &str) -> Result<()> {
    ensure!(
        bytes == id.bytes() && digest == id.sha256(),
        "Whisper model integrity check failed; remove and download it again"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_wrong_size_and_hash() {
        assert!(
            verify_digest(ModelId::Base, ModelId::Base.bytes(), ModelId::Base.sha256()).is_ok()
        );
        assert!(verify_digest(ModelId::Base, 2, ModelId::Base.sha256()).is_err());
        assert!(verify_digest(ModelId::Base, ModelId::Base.bytes(), "corrupt").is_err());
    }

    #[tokio::test]
    async fn active_model_cannot_be_removed_or_replaced() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(ModelId::Tiny.file()), b"not a model").unwrap();
        let manager = ModelManager::default();
        let lease = manager.lease(dir.path(), ModelId::Tiny).unwrap();
        assert!(lease.read_verified().is_err());
        assert!(manager.remove(dir.path(), ModelId::Tiny).is_err());
        assert!(manager.download(dir.path(), ModelId::Tiny).await.is_err());
        drop(lease);
        manager.remove(dir.path(), ModelId::Tiny).unwrap();
    }
}
