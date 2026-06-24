use anyhow;
#[cfg(windows)]
use std::path::Path;
use std::path::PathBuf;

use crate::rime_levers::*;
#[cfg(windows)]
use std::ffi::OsStr;
#[cfg(windows)]
use windows::Win32::System::SystemInformation::{
    GetNativeSystemInfo, PROCESSOR_ARCHITECTURE, PROCESSOR_ARCHITECTURE_AMD64,
    PROCESSOR_ARCHITECTURE_ARM64, SYSTEM_INFO,
};
#[cfg(windows)]
use windows_version::OsVersion;
#[cfg(windows)]
use winreg::RegKey;

#[cfg(windows)]
pub fn 路徑相同(左: &Path, 右: &Path) -> bool {
    let 左標準 = 左.canonicalize().unwrap_or_else(|_| 左.to_path_buf());
    let 右標準 = 右.canonicalize().unwrap_or_else(|_| 右.to_path_buf());
    左標準.to_string_lossy().to_ascii_lowercase() == 右標準.to_string_lossy().to_ascii_lowercase()
}

#[cfg(windows)]
fn 檢查架構(arch: PROCESSOR_ARCHITECTURE) -> bool {
    let mut info = SYSTEM_INFO::default();
    unsafe {
        GetNativeSystemInfo(&mut info);
        info.Anonymous.Anonymous.wProcessorArchitecture == arch
    }
}

#[cfg(windows)]
fn 系統是amd64架構() -> bool {
    檢查架構(PROCESSOR_ARCHITECTURE_AMD64)
}

#[cfg(windows)]
fn 系統是arm64架構() -> bool {
    檢查架構(PROCESSOR_ARCHITECTURE_ARM64)
}

#[cfg(windows)]
fn 版本高於_win11() -> bool {
    let 系統版本 = OsVersion::current();
    系統版本.major > 10 && 系統版本.build >= 22000
}

#[cfg(windows)]
pub fn 獲取小狼毫架構模式() -> String {
    if 版本高於_win11() {
        if 系統是arm64架構() || 系統是amd64架構() {
            "x64".to_string()
        } else {
            "x86".to_string()
        }
    } else {
        if 系統是amd64架構() {
            "x64".to_string()
        } else {
            "x86".to_string()
        }
    }
}

#[cfg(windows)]
pub fn 獲取小狼毫程序目錄() -> Option<String> {
    let 註冊表路徑 = {
        if 系統是arm64架構() || 系統是amd64架構() {
            OsStr::new("SOFTWARE\\WOW6432Node\\Rime\\Weasel")
        } else {
            OsStr::new("SOFTWARE\\Rime\\Weasel")
        }
    };
    RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE)
        .open_subkey(註冊表路徑)
        .and_then(|注冊表鍵| 注冊表鍵.get_value("WeaselRoot"))
        .ok()
}

#[cfg(windows)]
pub fn 用戶目錄() -> Option<String> {
    let 註冊表路徑 = OsStr::new("SOFTWARE\\Rime\\Weasel");
    RegKey::predef(winreg::enums::HKEY_CURRENT_USER)
        .open_subkey(註冊表路徑)
        .and_then(|注冊表鍵| 注冊表鍵.get_value("RimeUserDir"))
        .ok()
}

#[cfg(windows)]
pub fn 共享數據目錄() -> Option<String> {
    let 程序目錄 = 獲取小狼毫程序目錄()?;
    let mut 路徑 = PathBuf::from(程序目錄);
    路徑.push("data");
    Some(路徑.to_string_lossy().to_string())
}

#[cfg(windows)]
pub fn 默認用戶目錄() -> Option<String> {
    if let Some(家目錄) = std::env::var_os("APPDATA") {
        let mut 路徑 = std::path::PathBuf::from(家目錄);
        路徑.push("Rime");
        Some(路徑.to_string_lossy().to_string())
    } else {
        None
    }
}

#[cfg(windows)]
pub fn 卸載引擎庫() -> anyhow::Result<()> {
    use rime::dynload;
    if rime::IS_DYNAMIC_LOAD {
        rime::unload_librime!();
    }
    Ok(())
}

#[cfg(windows)]
pub fn 需要管理員權限(目錄: &str) -> anyhow::Result<bool> {
    let 程序目錄 = Path::new(目錄);
    let 測試文件 = 程序目錄.join("rime_cli_test_write.txt");
    match std::fs::File::create(&測試文件) {
        Ok(_) => {
            std::fs::remove_file(&測試文件).ok();
            Ok(false)
        }
        Err(_) => Ok(true),
    }
}

#[cfg(not(windows))]
pub fn 用戶目錄() -> Option<String> {
    if let Some(家目錄) = std::env::var_os("HOME") {
        let mut 路徑 = std::path::PathBuf::from(家目錄);

        #[cfg(target_os = "macos")]
        路徑.push("Library/Rime");
        #[cfg(not(target_os = "macos"))]
        路徑.push(".config/ibus/rime");

        Some(路徑.to_string_lossy().to_string())
    } else {
        todo!("家路徑異常");
    }
}

#[cfg(not(windows))]
pub fn 默認用戶目錄() -> Option<String> {
    用戶目錄()
}

#[cfg(not(windows))]
pub fn 共享數據目錄() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        if let Some(目標路徑) = std::env::var_os("DSTROOT") {
            let mut 路徑 = std::path::PathBuf::from(目標路徑);
            路徑.push("Contents/SharedSupport");
            Some(路徑.to_string_lossy().to_string())
        } else {
            todo!("DSTROOT路徑異常")
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        Some("/usr/share/rime-data".to_string())
    }
}

pub fn 前端部署() -> anyhow::Result<()> {
    #[cfg(windows)]
    {
        let 小狼毫目錄 =
            獲取小狼毫程序目錄().ok_or_else(|| anyhow::anyhow!("無法獲取小狼毫程序目錄"))?;
        let 服務 = PathBuf::from(小狼毫目錄).join("WeaselDeployer.exe");
        if !服務.exists() {
            return Err(anyhow::anyhow!("無法找到 WeaselDeployer.exe"));
        }
        std::process::Command::new(服務).arg("/deploy").spawn()?;
    }
    #[cfg(not(windows))]
    {
        //todo!("實現非 Windows 平台的前端部署");
        crate::rime_levers::製備輸入法固件()?
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct 啓動上下文 {
    pub 用戶數據目錄: PathBuf,
    pub 共享數據目錄: Option<PathBuf>,
    pub 日誌目錄: Option<PathBuf>,
    pub 應用名: Option<String>,
    pub 發行品名: Option<String>,
    pub 發行代號: Option<String>,
    pub 發行版本: Option<String>,
    pub 最小日誌級別: Option<i32>,
    pub 預構建數據目錄: Option<PathBuf>,
    pub 緩存目錄: Option<PathBuf>,
}

#[derive(Debug, Default)]
pub struct 啓動上下文覆蓋 {
    pub 用戶數據目錄: Option<PathBuf>,
    pub 共享數據目錄: Option<PathBuf>,
    pub 日誌目錄: Option<PathBuf>,
    pub 應用名: Option<String>,
    pub 發行品名: Option<String>,
    pub 發行代號: Option<String>,
    pub 發行版本: Option<String>,
    pub 最小日誌級別: Option<i32>,
    pub 預構建數據目錄: Option<PathBuf>,
    pub 緩存目錄: Option<PathBuf>,
}

impl 啓動上下文 {
    pub fn 解析(覆蓋: 啓動上下文覆蓋) -> anyhow::Result<Self> {
        let 用戶數據目錄 = 覆蓋
            .用戶數據目錄
            .or_else(|| 用戶目錄().map(PathBuf::from))
            .ok_or_else(|| anyhow::anyhow!("無法獲取用戶目錄"))?;
        let 共享數據目錄 = 覆蓋
            .共享數據目錄
            .or_else(|| 共享數據目錄().map(PathBuf::from));
        let 日誌目錄 = 覆蓋.日誌目錄.or_else(|| {
            std::env::var_os("HOME")
                .or_else(|| std::env::var_os("USERPROFILE"))
                .map(|家目錄| PathBuf::from(家目錄).join(".rime-cli").join("logs"))
        });
        Ok(Self {
            用戶數據目錄,
            共享數據目錄,
            日誌目錄,
            應用名: 覆蓋.應用名.or_else(|| Some("rime-cli".to_string())),
            發行品名: 覆蓋.發行品名,
            發行代號: 覆蓋.發行代號,
            發行版本: 覆蓋.發行版本,
            最小日誌級別: 覆蓋.最小日誌級別,
            預構建數據目錄: 覆蓋.預構建數據目錄,
            緩存目錄: 覆蓋.緩存目錄,
        })
    }
}

pub fn 初始化引擎(上下文: &啓動上下文) -> anyhow::Result<()> {
    let mut 參數 = 引擎啓動參數::新建(上下文.用戶數據目錄.clone());
    參數.共享數據場地 = 上下文.共享數據目錄.clone();
    參數.日誌場地 = 上下文.日誌目錄.clone();
    參數.應用名 = 上下文.應用名.clone();
    參數.品名 = 上下文.發行品名.clone();
    參數.代號 = 上下文.發行代號.clone();
    參數.版本 = 上下文.發行版本.clone();
    參數.最小日誌級別 = 上下文.最小日誌級別;
    參數.預構建固件場地 = 上下文.預構建數據目錄.clone();
    參數.緩存場地 = 上下文.緩存目錄.clone();
    crate::rime_levers::設置引擎啓動參數(&參數)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[cfg(windows)]
    use super::*;

    #[test]
    #[cfg(windows)]
    fn test_路徑相同() {
        let p1 = Path::new("C:\\Windows\\System32\\notepad.exe");
        let p2 = Path::new("C:\\Windows\\System32\\..\\System32\\notepad.exe");
        assert!(路徑相同(p1, p2));
    }
    #[test]
    #[cfg(windows)]
    fn test_默认用戶目錄() {
        let 目錄 = 默認用戶目錄().unwrap();
        let 路徑 = Path::new(&目錄);
        let 家 = std::env::var_os("USERPROFILE").unwrap();
        let 預期路徑 = Path::new(&家).join("AppData\\Roaming\\Rime");
        assert_eq!(路徑, 預期路徑);
    }
}
