use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::OnceLock};
use zed_extension_api::{self as zed, Architecture, http_client::*, serde_json};

#[derive(Default)]
pub(crate) struct MonoDebugEntry {
    cached_path: OnceLock<String>,
}

impl MonoDebugEntry {
    pub fn new() -> Self {
        MonoDebugEntry {
            cached_path: OnceLock::new(),
        }
    }

    pub fn get_binary_path(&self) -> Result<String, String> {
        if let Some(path) = self.cached_path.get() {
            return Ok(path.clone());
        }

        if let Ok(path) = self.check_binary_exists() {
            return Ok(path);
        }

        self.download_vsx()
    }

    fn download_vsx(&self) -> Result<String, String> {
        let (platform, arch) = zed::current_platform();

        if platform != zed::Os::Windows {
            return Err("Unsupported platform".to_string());
        }

        let vsx_arch = match arch {
            Architecture::X8664 => "win32-x64",
            Architecture::Aarch64 => "win32-arm64",
            _ => return Err("Unsupported architecture".to_string()),
        };

        let response = HttpRequestBuilder::new()
            .method(HttpMethod::Get)
            .url("https://open-vsx.org/api/nromanov/dotrush/latest")
            .build()?
            .fetch();

        if let Err(r) = response {
            return Err(format!("Failed to fetch from open VSX: {}", r));
        }

        let download_url = response
            .and_then(|r| {
                let info: VsxInfo = serde_json::from_slice(&r.body).unwrap_or(VsxInfo::default());

                info.downloads
                    .get(vsx_arch)
                    .ok_or(format!("No download URL found"))
                    .map(|url| url.to_string())
            })
            .map_err(|e| format!("Failed to get VSX info: {}", e))?;

        let file_type = zed::DownloadedFileType::Zip;

        zed::download_file(&download_url, "./Dotrush", file_type)
            .and_then(|_| self.check_binary_exists())
    }

    fn check_binary_exists(&self) -> Result<String, String> {
        let binary_path = std::env::current_dir()
            .map_err(|e| format!("Cannot get current directory: {}", e))?
            .join("Dotrush/extension/extension/bin/DebuggerMono/monodbg.exe");

        if binary_path.exists() {
            let mut path = binary_path.to_string_lossy().to_string();
            // remove the weird directory slash at beginning of path
            path.remove(0);
            self.cached_path.set(path.clone())?;
            Ok(path)
        } else {
            Err(format!(
                "Binary does not exist at {}",
                binary_path.to_string_lossy()
            ))
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct VsxInfo {
    #[serde(default, rename = "downloads")]
    downloads: HashMap<String, String>,
}
