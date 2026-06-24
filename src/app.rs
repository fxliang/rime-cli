use structopt::StructOpt;

use crate::client::*;
use crate::download::{下載參數, 下載配方包};
use crate::install::安裝配方;
use crate::recipe::配方名片;
use crate::rime_levers::{
    加入輸入方案列表, 從方案列表中刪除, 檢查默認設置自定義文件, 製備輸入法固件, 選擇輸入方案,
    配置補丁,
};

#[derive(Debug, StructOpt)]
pub enum 子命令 {
    /// 加入輸入方案列表
    Add {
        /// 要向列表中追加的輸入方案
        schemata: Vec<String>,
    },
    /// 從方案列表中刪除
    Remove {
        /// 要從列表中刪除的輸入方案
        schemata: Vec<String>,
    },
    /// 構建輸入法固件
    Build,
    /// 部署輸入法固件到目標位置
    Deploy,
    /// 下載配方包
    Download {
        /// 要下載的配方包
        recipes: Vec<String>,
        #[structopt(flatten)]
        下載參數: 下載參數,
    },
    /// 安裝配方
    Install {
        /// 要安裝的配方
        recipes: Vec<String>,
        #[structopt(flatten)]
        下載參數: 下載參數,
    },
    /// 更新引擎庫
    Get {
        /// 目標版本標籤，如1.15.0（留空表示最新）
        tag: Option<String>,
        #[structopt(flatten)]
        下載參數: 下載參數,
    },
    /// 新建配方
    New {
        /// 配方名字
        _name: Option<String>,
    },
    /// 配置補丁
    Patch {
        /// 目標配置
        config: String,
        /// 紐
        key: String,
        /// 值
        value: String,
    },
    /// 選擇輸入方案
    Select {
        /// 選中的輸入方案
        schema: String,
    },
    #[cfg(feature = "tui")]
    /// 進入互動式界面
    Tui,
}

pub fn 執行命令(
    上下文: &啓動上下文, 命令行參數: 子命令, 圖形界面: bool
) -> anyhow::Result<()> {
    match 命令行參數 {
        子命令::Add { schemata } => {
            if !圖形界面 {
                初始化引擎(上下文)?;
                檢查默認設置自定義文件();
            }

            加入輸入方案列表(&schemata)?;
            前端部署()?;
            return Ok(());
        }
        子命令::Remove { schemata } => {
            if !圖形界面 {
                初始化引擎(上下文)?;
                檢查默認設置自定義文件();
            }
            從方案列表中刪除(&schemata)?;
            前端部署()?;
            return Ok(());
        }
        子命令::Build => {
            #[cfg(windows)]
            {
                前端部署()?;
                return Ok(());
            }
            #[cfg(not(windows))]
            {
                if !圖形界面 {
                    初始化引擎(上下文)?;
                }
                製備輸入法固件()?;
            }
        }
        子命令::Download {
            recipes, 下載參數
        } => {
            let 衆配方 = recipes
                .iter()
                .map(|rx| 配方名片::from(rx.as_str()))
                .collect::<Vec<_>>();
            下載配方包(&衆配方, 下載參數)?;
        }
        子命令::Install {
            recipes, 下載參數
        } => {
            let 衆配方 = recipes
                .iter()
                .map(|rx| 配方名片::from(rx.as_str()))
                .collect::<Vec<_>>();
            下載配方包(&衆配方, 下載參數)?;

            for 配方 in &衆配方 {
                安裝配方(配方, &上下文.用戶數據目錄)?;
            }
        }
        子命令::Patch { config, key, value } => {
            if !圖形界面 {
                初始化引擎(上下文)?;
                檢查默認設置自定義文件();
            }

            配置補丁(&config, &key, &value)?;
            製備輸入法固件()?;
        }
        子命令::Select { schema } => {
            if !圖形界面 {
                初始化引擎(上下文)?;
                檢查默認設置自定義文件();
            }

            選擇輸入方案(&schema)?;
        }
        子命令::Get { tag, 下載參數 } => {
            #[cfg(windows)]
            {
                use std::os::windows::ffi::OsStrExt;
                use windows::core::PCWSTR;
                use windows::Win32::UI::Shell::ShellExecuteW;
                use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

                let 程序目錄 = 獲取小狼毫程序目錄()
                    .ok_or_else(|| anyhow::anyhow!("无法获取小狼毫程序目录"))?;
                if crate::client::需要管理員權限(&程序目錄)? {
                    let exe_path = std::env::current_exe()?.to_string_lossy().to_string();
                    let mut args_vec = vec!["get".to_string()];
                    if let Some(t) = tag {
                        args_vec.push(t);
                    }
                    if let Some(h) = 下載參數.倉庫域名() {
                        args_vec.push("--host".to_string());
                        args_vec.push(h.to_string());
                    }
                    if let Some(p) = 下載參數.代理地址() {
                        args_vec.push("--proxy".to_string());
                        args_vec.push(p.to_string());
                    }
                    let args_str = args_vec.join(" ");

                    log::debug!("elevate: exe={}, args={}", exe_path, args_str);
                    let exe_w: Vec<u16> = std::ffi::OsStr::new(&exe_path)
                        .encode_wide()
                        .chain(std::iter::once(0))
                        .collect();
                    let args_w: Vec<u16> = std::ffi::OsStr::new(&args_str)
                        .encode_wide()
                        .chain(std::iter::once(0))
                        .collect();

                    unsafe {
                        let res = ShellExecuteW(
                            None,
                            windows::core::w!("runas"),
                            PCWSTR::from_raw(exe_w.as_ptr()),
                            PCWSTR::from_raw(args_w.as_ptr()),
                            PCWSTR::null(),
                            SW_SHOWNORMAL,
                        );
                        let code = res.0 as isize;
                        if code <= 32 {
                            eprintln!("ShellExecuteW failed: {}", code);
                        } else {
                            log::debug!("elevate: ShellExecuteW ok: {}", code);
                        }
                    }
                    return Ok(());
                }
            }
            let 目標版本 = tag.unwrap_or("".to_string());
            crate::get_rime::更新引擎庫(上下文, &目標版本, &下載參數)?;
        }
        #[cfg(feature = "tui")]
        子命令::Tui => {}
        _ => todo!("還沒做呢"),
    }

    Ok(())
}
