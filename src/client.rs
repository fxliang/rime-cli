use std::path::PathBuf;
#[cfg(windows)]
use std::path::Path;
#[cfg(windows)]
use std::process::Command;
use anyhow;

#[cfg(windows)]
use std::ffi::OsStr;
#[cfg(windows)]
use windows::Win32::System::SystemInformation::{
    GetNativeSystemInfo, PROCESSOR_ARCHITECTURE, PROCESSOR_ARCHITECTURE_AMD64, PROCESSOR_ARCHITECTURE_ARM64, SYSTEM_INFO
};
#[cfg(windows)]
use windows_version::OsVersion;
#[cfg(windows)]
use winreg::RegKey;
use crate::rime_levers::{*};

#[cfg(windows)]
fn 查找_powershell() -> Option<PathBuf> {
    let mut 候選: Vec<PathBuf> = Vec::new();
    if let Some(root) = std::env::var_os("SystemRoot") {
        let mut p = PathBuf::from(root);
        p.push("System32\\WindowsPowerShell\\v1.0\\powershell.exe");
        候選.push(p);
    }
    候選.push(PathBuf::from("powershell.exe"));
    候選.push(PathBuf::from("pwsh.exe"));
    if let Some(path_var) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path_var) {
            候選.push(dir.join("powershell.exe"));
            候選.push(dir.join("pwsh.exe"));
        }
    }
    候選.into_iter().find(|p| p.is_file())
}

#[cfg(windows)]
pub struct 提權複製選項 {
    pub 隱藏窗口: bool,
    pub 等待完成: bool,
    pub 父進程: Option<u32>,
    pub 重啟服務: Option<PathBuf>,
    pub 驗證哈希: bool,
}

#[cfg(windows)]
pub fn 執行提權複製腳本(來源: &Path, 目標: &Path, 選項: 提權複製選項) -> anyhow::Result<()> {
    if !來源.exists() {
        anyhow::bail!(format!("源文件不存在: {}", 來源.display()));
    }
    let ps = 查找_powershell().ok_or_else(|| anyhow::anyhow!("未找到 PowerShell，可手動複製 rime.dll"))?;
    let 來源路徑 = 來源.canonicalize()
        .unwrap_or_else(|_| 來源.to_path_buf())
        .to_string_lossy()
        .replace("'", "''");
    let 目標路徑 = 目標.canonicalize()
        .unwrap_or_else(|_| 目標.to_path_buf())
        .to_string_lossy()
        .replace("'", "''");

    let 哈希函數 = if 選項.驗證哈希 {
        "function Hash($p){ $sha=[System.Security.Cryptography.SHA256]::Create(); $fs=[System.IO.File]::OpenRead($p); try { ($sha.ComputeHash($fs) | ForEach-Object { $_.ToString('x2') }) -join '' } finally { $fs.Dispose(); $sha.Dispose() } };"
    } else { "" };

    let 等待父進程 = match 選項.父進程 {
        Some(pid) => format!("Wait-Process -Id {} -ErrorAction SilentlyContinue; ", pid),
        None => "".to_string(),
    };

    let 驗證腳本 = if 選項.驗證哈希 {
        "\
        $srcHash = Hash $source; \
        $dstHash = Hash $dest; \
        if ($srcHash -ne $dstHash) { throw \"hash mismatch src=$srcHash dst=$dstHash\" };"
    } else {
        ""
    };

    let 重啟服務腳本 = if let Some(svc) = 選項.重啟服務 {
        let svc_path = svc.to_string_lossy().replace("'", "''");
        // 隐藏启动 WeaselServer，短等待后检测进程是否存在，若不存在则改为可见模式重试；同时在同目录执行 WeaselDeployer.exe /deploy（若存在）。
        format!(
            " if (Test-Path '{svc}') {{ \n\
                $dir = Split-Path -Parent '{svc}'; \n\
                $name = [System.IO.Path]::GetFileNameWithoutExtension('{svc}'); \n\
                Start-Process -FilePath '{svc}' -WorkingDirectory $dir -WindowStyle Hidden | Out-Null; \n\
                Start-Sleep -Milliseconds 500; \n\
                if (-not (Get-Process -Name $name -ErrorAction SilentlyContinue)) {{ \n\
                    Start-Process -FilePath '{svc}' -WorkingDirectory $dir -WindowStyle Normal | Out-Null; \n\
                }}; \n\
                $deployer = Join-Path $dir 'WeaselDeployer.exe'; \n\
                if (Test-Path $deployer) {{ \n\
                    $psi = New-Object System.Diagnostics.ProcessStartInfo; \n\
                    $psi.FileName = $deployer; \n\
                    $psi.Arguments = '/deploy'; \n\
                    $psi.WorkingDirectory = $dir; \n\
                    $proc = [System.Diagnostics.Process]::Start($psi); \n\
                    if ($proc) {{ $proc.WaitForExit(); }} \n\
                }}; \n\
            }};",
            svc = svc_path,
        )
    } else {
        "".to_string()
    };

    

    let 內層腳本 = format!(
        "\
        $ErrorActionPreference='Stop'; \
        {hash_fn} \
        $source='{source}'; \
        $dest='{dest}'; \
        {wait_parent}New-Item -ItemType Directory -Force -Path (Split-Path -Parent $dest) | Out-Null; \
        Copy-Item -LiteralPath $source -Destination $dest -Force;{verify}{restart}",
        hash_fn = 哈希函數,
        source = 來源路徑,
        dest = 目標路徑,
        wait_parent = 等待父進程,
        verify = 驗證腳本,
        restart = 重啟服務腳本,
    );

    let 內層腳本_轉義 = 內層腳本.replace("'", "''");
    let ps_cmd = ps.to_string_lossy().replace("'", "''");
    let window_flag = if 選項.隱藏窗口 { "-WindowStyle Hidden " } else { "" };
    let wait_flag = if 選項.等待完成 { " -Wait" } else { "" };
    // 如果目標路徑不需要提權，則不需要提權啟動
    let 需要提權 = 目標.metadata().map(|m| m.permissions().readonly()).unwrap_or(true);
    let 提權命令 = if 需要提權 {
        format!(
            "Start-Process -FilePath '{ps}' -Verb RunAs {window}-ArgumentList '-NoProfile','-ExecutionPolicy','Bypass','-Command','{script}'{wait}",
            ps = ps_cmd,
            window = window_flag,
            script = 內層腳本_轉義,
            wait = wait_flag,
        )
    } else {
        // 不需要提權，直接執行 PowerShell 命令
        format!(
            "& '{ps}' -NoProfile -ExecutionPolicy Bypass -Command '{script}'",
            ps = ps_cmd,
            script = 內層腳本_轉義,
        )
    };

    if 選項.等待完成 {
        let 狀態 = Command::new(&ps)
            .arg("-NoProfile")
            .arg("-ExecutionPolicy").arg("Bypass")
            .arg("-Command").arg(提權命令)
            .status()?;
        if !狀態.success() {
            anyhow::bail!("提權腳本執行失敗，請允許提權或手動複製。");
        }
    } else {
        Command::new(&ps)
            .arg("-NoProfile")
            .arg("-ExecutionPolicy").arg("Bypass")
            .arg("-Command").arg(提權命令)
            .spawn()?;
    }

    Ok(())
}

#[cfg(windows)]
pub fn 路徑相同(左: &Path, 右: &Path) -> bool {
    let 左標準 = 左.canonicalize().unwrap_or_else(|_| 左.to_path_buf());
    let 右標準 = 右.canonicalize().unwrap_or_else(|_| 右.to_path_buf());
    左標準.to_string_lossy().to_ascii_lowercase()
        == 右標準.to_string_lossy().to_ascii_lowercase()
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
fn 系統是amd64架構() -> bool { 檢查架構(PROCESSOR_ARCHITECTURE_AMD64) }

#[cfg(windows)]
fn 系統是arm64架構() -> bool { 檢查架構(PROCESSOR_ARCHITECTURE_ARM64) }

#[cfg(windows)]
fn 版本高於_win11() -> bool {
    let 系統版本 = OsVersion::current();
    系統版本.major > 10 && 系統版本.build >= 22000
}

#[cfg(windows)]
pub fn 獲取小狼毫架構模式() -> String {
    if 版本高於_win11() {
        if 系統是arm64架構() || 系統是amd64架構() {"x64".to_string()}
        else { "x86".to_string() }
    } else {
        if 系統是amd64架構() {"x64".to_string()}
        else { "x86".to_string() }
    }
}

#[cfg(windows)]
pub fn 獲取小狼毫程序目錄() -> Option<String> {
    let 註冊表路徑 = {
        if 系統是arm64架構() || 系統是amd64架構() { OsStr::new("SOFTWARE\\WOW6432Node\\Rime\\Weasel") }
        else { OsStr::new("SOFTWARE\\Rime\\Weasel") }
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
pub fn 共享數據目錄() ->Option<String> {
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
pub fn 共享數據目錄() ->Option<String> {
    #[cfg(target_os = "macos")] {
        if let Some(目標路徑) = std::env::var_os("DSTROOT") {
            let mut 路徑 = std::path::PathBuf::from(目標路徑);
            路徑.push("Contents/SharedSupport");
            Some(路徑.to_string_lossy().to_string())
        } else {
            todo!("DSTROOT路徑異常")
        }
    }
    #[cfg(not(target_os = "macos"))] {
        Some("/usr/share/rime-data".to_string())
    }
}

pub fn 前端部署() -> anyhow::Result<()> {
    #[cfg(windows)]
    {
        let 小狼毫目錄 = 獲取小狼毫程序目錄().ok_or_else(|| anyhow::anyhow!("無法獲取小狼毫程序目錄"))?;
        let 服務 = PathBuf::from(小狼毫目錄).join("WeaselDeployer.exe");
        if !服務.exists() {
            return Err(anyhow::anyhow!("無法找到 WeaselDeployer.exe"));
        }
        std::process::Command::new(服務)
            .arg("/deploy")
            .spawn()?;
    }
    #[cfg(not(windows))]
    {
        //todo!("實現非 Windows 平台的前端部署");
        crate::rime_levers::製備輸入法固件()?
    }
    Ok(())
}

pub fn 初始化引擎() -> anyhow::Result<()> {
    let 用戶數據目錄 = 用戶目錄().map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    let mut 參數 = 引擎啓動參數::新建(用戶數據目錄);
    參數.共享數據場地 = 共享數據目錄().map(PathBuf::from);
    let 家目錄 = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"));
    let 日誌目錄 = PathBuf::from(家目錄.unwrap()).join(".rime-cli").join("logs");
    參數.日誌場地 = Some(日誌目錄);
    參數.應用名 = Some("rime-cli".to_string());
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
