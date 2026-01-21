use crate::utils;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf, sync::OnceLock};
use zed_extension_api::{self as zed, Architecture, http_client::*, serde_json};

const BIN_NAME: &str = "monodbg.exe";

pub(crate) struct MonoDebugEntry {
    cached_path: OnceLock<String>,
}

impl Default for MonoDebugEntry {
    fn default() -> Self {
        MonoDebugEntry {
            cached_path: OnceLock::new(),
        }
    }
}

impl MonoDebugEntry {
    pub fn get_binary_path(&self) -> Result<String, String> {
        let platform_info = self.get_platform_info()?;

        if let Some(path) = self.cached_path.get() {
            return Ok(path.clone());
        }

        if let Ok(path) = self.check_binary_exists(self.default_abs_binary_path(platform_info)?) {
            return Ok(path);
        }

        self.get_vsx_info()
            .and_then(|info| self.download_vsx(&info, platform_info))
    }

    fn get_platform_info(&self) -> Result<&str, String> {
        let (platform, arch) = zed::current_platform();

        if platform != zed::Os::Windows {
            return Err("Unsupported platform".to_string());
        }

        match arch {
            Architecture::X8664 => Ok("win32-x64"),
            Architecture::Aarch64 => Ok("win32-arm64"),
            _ => return Err("Unsupported architecture".to_string()),
        }
    }

    fn get_vsx_info(&self) -> Result<VsxInfo, String> {
        let response = HttpRequestBuilder::new()
            .method(HttpMethod::Get)
            .url("https://open-vsx.org/api/nromanov/dotrush/latest")
            .build()?
            .fetch();

        response
            .map_err(|e| format!("Failed to fetch from open VSX: {}", e))
            .and_then(|r| {
                serde_json::from_slice(&r.body)
                    .map_err(|e| format!("Failed deserialize vsx body json: {}", e))
            })
    }

    fn download_vsx(&self, info: &VsxInfo, arch: &str) -> Result<String, String> {
        let download_url = info.downloads.get(arch).ok_or("No download URL found")?;
        let file_type = zed::DownloadedFileType::Zip;
        zed::download_file(&download_url, "./temp", file_type)?;
        fs::create_dir_all("./bin")
            .map_err(|e| format!("Failed to create binary directory: {}", e))?;
        fs::rename(
            "./temp/extension/extension/bin/DebuggerMono",
            format!("./bin/{}", arch),
        )
        .map_err(|e| format!("Failed to move binary directory: {}", e))?;

        // Special implementation for remove temp folder
        utils::remove_dir("./temp")
            .map_err(|e| format!("Failed to remove temp directory: {}", e))?;

        self.check_binary_exists(self.default_abs_binary_path(arch)?)
    }

    fn check_binary_exists(&self, binary_path: PathBuf) -> Result<String, String> {
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

    fn default_abs_binary_path(&self, arch: &str) -> Result<PathBuf, String> {
        std::env::current_dir()
            .map_err(|e| format!("Cannot get current directory: {}", e))
            .map(|buf| buf.join(self.default_rel_binary_path(arch)))
    }

    fn default_rel_binary_path(&self, arch: &str) -> PathBuf {
        PathBuf::from(format!("./bin/{}/{}", arch, BIN_NAME))
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct VsxInfo {
    #[serde(default)]
    version: String,
    #[serde(default, rename = "downloads")]
    downloads: HashMap<String, String>,
}
