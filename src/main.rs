use dialoguer::{
    console::{style, Term},
    theme::ColorfulTheme,
    Input, Select,
};
use std::{fs, path::PathBuf};
use structopt::StructOpt;

mod deploy;
mod download;
mod get_rime;
mod install;
mod package;
mod recipe;
mod rime_levers;

use deploy::部署輸入法固件;
use download::{下載參數, 下載配方包};
use install::安裝配方;
use recipe::配方名片;
use rime_levers::{
    加入輸入方案列表, 測試輸入法固件, 製備輸入法固件, 設置引擎構建與整備參數, 選擇輸入方案,
    配置補丁,
};

#[derive(Debug, StructOpt)]
pub struct 場地參數 {
    /// 配方包下載緩存目錄
    #[structopt(
        long = "download-dir",
        env = "DOWNLOAD_DIR",
        default_value = "pkg",
        value_name = "DIR",
        global = true
    )]
    pub 下載場地: PathBuf,
    /// 編譯期源文件目錄
    #[structopt(
        long = "source-dir",
        env = "SOURCE_DIR",
        default_value = "src",
        value_name = "DIR",
        global = true
    )]
    pub 源碼場地: PathBuf,
    /// 運行時產物整備目錄
    #[structopt(
        long = "staging-dir",
        env = "STAGING_DIR",
        default_value = "stage",
        value_name = "DIR",
        global = true
    )]
    pub 整備場地: PathBuf,
    /// 部署目標目錄
    #[structopt(
        long = "target-dir",
        env = "TARGET_DIR",
        default_value = "target",
        value_name = "DIR",
        global = true
    )]
    pub 目標場地: PathBuf,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "rime", about = "Rime 配方管理器")]
struct 命令行參數 {
    #[structopt(flatten)]
    場地: 場地參數,
    #[structopt(subcommand)]
    命令: 子命令,
}

#[derive(Debug, StructOpt)]
enum 子命令 {
    /// 加入輸入方案列表
    Add {
        /// 要向列表中追加的輸入方案
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
    /// 更新引擎庫
    Get {
        tag: Option<String>,
        #[structopt(flatten)]
        下載參數: 下載參數,
    },
    /// 安裝配方
    Install {
        /// 跳過下載步驟，直接從本地緩存倉庫安裝
        #[structopt(long = "no-download")]
        不下載: bool,
        /// 要安裝的配方
        recipes: Vec<String>,
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
    /// 在整備區測試構建好的輸入法數據
    Test {
        /// 要測試的輸入方案（留空則測試全部已構建方案）
        schemata: Vec<String>,
    },
    /// 進入互動式界面
    Tui,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let 參數 = 命令行參數::from_args();
    log::debug!("參數: {:?}", 參數);
    let 場地 = &參數.場地;

    執行命令(參數.命令, 場地)?;
    Ok(())
}

fn 執行命令(命令行參數: 子命令, 場地: &場地參數) -> anyhow::Result<()> {
    match 命令行參數 {
        子命令::Add { schemata } => {
            設置引擎構建與整備參數(&場地.源碼場地, &場地.整備場地)?;
            加入輸入方案列表(&schemata)?;
        }
        子命令::Build => {
            設置引擎構建與整備參數(&場地.源碼場地, &場地.整備場地)?;
            製備輸入法固件()?;
        }
        子命令::Deploy => {
            部署輸入法固件(&場地.整備場地, &場地.目標場地)?;
        }
        子命令::Download {
            recipes, 下載參數
        } => {
            let 衆配方 = recipes
                .iter()
                .map(|rx| 配方名片::from(rx.as_str()))
                .collect::<Vec<_>>();
            下載配方包(&衆配方, &場地.下載場地, &下載參數)?;
        }
        子命令::Get { tag, 下載參數 } => {
            let 目標版本 = tag.unwrap_or_default();
            get_rime::更新引擎庫(&目標版本, &下載參數)?;
        }
        子命令::Install {
            不下載,
            recipes,
            下載參數,
        } => {
            let 衆配方 = recipes
                .iter()
                .map(|rx| 配方名片::from(rx.as_str()))
                .collect::<Vec<_>>();
            if !不下載 {
                下載配方包(&衆配方, &場地.下載場地, &下載參數)?;
            }
            for 配方 in &衆配方 {
                安裝配方(配方, &場地.下載場地, &場地.源碼場地, &場地.整備場地)?;
            }
        }
        子命令::Patch { config, key, value } => {
            設置引擎構建與整備參數(&場地.源碼場地, &場地.整備場地)?;
            配置補丁(&config, &key, &value)?;
        }
        子命令::Select { schema } => {
            設置引擎構建與整備參數(&場地.源碼場地, &場地.整備場地)?;
            選擇輸入方案(&schema)?;
        }
        子命令::Test { schemata } => {
            測試輸入法固件(&場地.整備場地, &schemata)?;
        }
        子命令::Tui => 進入tui(場地)?,
        _ => todo!("還沒做呢"),
    }

    Ok(())
}

#[derive(Default, Clone)]
struct Tui配置 {
    proxy: Option<String>,
    host: Option<String>,
}

fn 配置文件路徑() -> Option<PathBuf> {
    let 家目錄 = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"));
    家目錄.map(|家| {
        let mut 路徑 = PathBuf::from(家);
        路徑.push(".rime-cli");
        路徑.push("config");
        路徑
    })
}

fn 讀取tui配置() -> anyhow::Result<Tui配置> {
    let mut 配置 = Tui配置::default();
    if let Some(路徑) = 配置文件路徑() {
        if 路徑.exists() {
            if let Ok(內容) = fs::read_to_string(&路徑) {
                for 行 in 內容.lines() {
                    let mut 分隔 = 行.splitn(2, '=');
                    if let (Some(鍵), Some(值)) = (分隔.next(), 分隔.next()) {
                        let 內容 = 值.trim();
                        if 內容.is_empty() {
                            continue;
                        }
                        match 鍵.trim() {
                            "proxy" => 配置.proxy = Some(內容.to_string()),
                            "host" => 配置.host = Some(內容.to_string()),
                            _ => {}
                        }
                    }
                }
            }
        }
    }
    Ok(配置)
}

fn 保存tui配置(配置: &Tui配置) -> anyhow::Result<()> {
    if let Some(路徑) = 配置文件路徑() {
        if let Some(父目錄) = 路徑.parent() {
            fs::create_dir_all(父目錄)?;
        }
        let mut 內容 = String::new();
        if let Some(host) = &配置.host {
            內容.push_str(&format!("host={}\n", host));
        }
        if let Some(proxy) = &配置.proxy {
            內容.push_str(&format!("proxy={}\n", proxy));
        }
        fs::write(路徑, 內容)?;
    }
    Ok(())
}

fn 進入tui(場地: &場地參數) -> anyhow::Result<()> {
    let 主題 = ColorfulTheme::default();
    let 終端 = Term::stdout();
    let mut 配置 = 讀取tui配置()?;
    let mut proxy = 配置.proxy.clone();
    let mut host = 配置.host.clone();
    let mut 狀態: Option<String> = None;

    'tui: loop {
        if let Some(msg) = 狀態.take() {
            println!("{msg}");
        }
        let 選項 = vec![
            "下載配方".to_string(),
            "安裝配方".to_string(),
            "更新引擎庫".to_string(),
            "選擇輸入方案".to_string(),
            "加入輸入方案列表".to_string(),
            "配置補丁".to_string(),
            "構建輸入法固件".to_string(),
            format!("設置代理 ({})", proxy.as_deref().unwrap_or("未設置")),
            format!("設置域名 ({})", host.as_deref().unwrap_or("未設置")),
            "退出".to_string(),
        ];
        let _ = 終端.write_line(橫線().as_str());
        let sel = Select::with_theme(&主題)
            .items(&選項)
            .default(0)
            .interact_on(&終端)?;
        let 應退出 = match sel {
            0 => {
                let Some(輸入) = 讀取可取消("要下載的配方 (空格分隔)", &主題)?
                else {
                    continue 'tui;
                };
                let 配方 = 輸入
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>();
                if !配方.is_empty() {
                    let mut args = vec!["download".to_string()];
                    args.extend(配方);
                    狀態 = Some(執行tui命令參數(
                        args,
                        場地,
                        host.as_deref(),
                        proxy.as_deref(),
                    )?);
                }
                false
            }
            1 => {
                let Some(輸入) = 讀取可取消("要安裝的配方 (空格分隔)", &主題)?
                else {
                    continue 'tui;
                };
                let 配方 = 輸入
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>();
                if !配方.is_empty() {
                    let mut args = vec!["install".to_string()];
                    args.extend(配方);
                    狀態 = Some(執行tui命令參數(
                        args,
                        場地,
                        host.as_deref(),
                        proxy.as_deref(),
                    )?);
                }
                false
            }
            2 => {
                let tag = match 讀取可取消("版本標籤 (留空使用最新)", &主題)? {
                    Some(t) => t,
                    None => continue 'tui,
                };
                let mut args = vec!["get".to_string()];
                if !tag.trim().is_empty() {
                    args.push(tag);
                }
                狀態 = Some(執行tui命令參數(
                    args,
                    場地,
                    host.as_deref(),
                    proxy.as_deref(),
                )?);
                false
            }
            3 => {
                let Some(schema) = 讀取可取消("選擇的輸入方案", &主題)? else {
                    continue 'tui;
                };
                if !schema.trim().is_empty() {
                    狀態 = Some(執行tui命令參數(
                        vec!["select".to_string(), schema],
                        場地,
                        host.as_deref(),
                        proxy.as_deref(),
                    )?);
                }
                false
            }
            4 => {
                let Some(輸入) = 讀取可取消("要加入的輸入方案 (空格分隔)", &主題)?
                else {
                    continue 'tui;
                };
                let 方案 = 輸入
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>();
                if !方案.is_empty() {
                    let mut args = vec!["add".to_string()];
                    args.extend(方案);
                    狀態 = Some(執行tui命令參數(
                        args,
                        場地,
                        host.as_deref(),
                        proxy.as_deref(),
                    )?);
                }
                false
            }
            5 => {
                let config = match 讀取可取消("目標配置 (如 default)", &主題)? {
                    Some(c) => c,
                    None => continue 'tui,
                };
                let key = match 讀取可取消("紐 (如 patch/menu/page_size)", &主題)? {
                    Some(k) => k,
                    None => continue 'tui,
                };
                let value = match 讀取可取消("值 (YAML 格式)", &主題)? {
                    Some(v) => v,
                    None => continue 'tui,
                };
                if !(config.trim().is_empty() || key.trim().is_empty() || value.trim().is_empty()) {
                    狀態 = Some(執行tui命令參數(
                        vec!["patch".to_string(), config, key, value],
                        場地,
                        host.as_deref(),
                        proxy.as_deref(),
                    )?);
                }
                false
            }
            6 => {
                狀態 = Some(執行tui命令參數(
                    vec!["build".to_string()],
                    場地,
                    host.as_deref(),
                    proxy.as_deref(),
                )?);
                false
            }
            7 => {
                let 輸入: String = Input::with_theme(&主題)
                    .with_prompt("Proxy (留空清除)")
                    .allow_empty(true)
                    .interact_text()?;
                proxy = 非空或無(輸入);
                配置.proxy = proxy.clone();
                保存tui配置(&配置)?;
                false
            }
            8 => {
                let 輸入: String = Input::with_theme(&主題)
                    .with_prompt("Host (留空清除)")
                    .allow_empty(true)
                    .interact_text()?;
                host = 非空或無(輸入);
                配置.host = host.clone();
                保存tui配置(&配置)?;
                false
            }
            _ => true,
        };
        if 應退出 {
            break;
        }
    }

    保存tui配置(&配置)?;
    Ok(())
}

fn 執行tui命令參數(
    args: Vec<String>,
    場地: &場地參數,
    host: Option<&str>,
    proxy: Option<&str>,
) -> anyhow::Result<String> {
    if args.is_empty() {
        return Ok(String::new());
    }

    let mut 全部參數 = vec!["rime".to_string()];
    全部參數.extend(args);

    if let Some(子命令名稱) = 全部參數.get(1).map(String::as_str) {
        if matches!(子命令名稱, "download" | "install" | "get") {
            if let Some(host) = host {
                if !包含選項(&全部參數, "--host") && !包含選項(&全部參數, "-h") {
                    全部參數.push("--host".to_string());
                    全部參數.push(host.to_string());
                }
            }
            if let Some(proxy) = proxy {
                if !包含選項(&全部參數, "--proxy") && !包含選項(&全部參數, "-p") {
                    全部參數.push("--proxy".to_string());
                    全部參數.push(proxy.to_string());
                }
            }
        }
    }

    match 子命令::from_iter_safe(全部參數.clone()) {
        Ok(cmd) => match 執行命令(cmd, 場地) {
            Ok(()) => {
                let 描述 = if 全部參數.len() > 1 {
                    全部參數[1..].join(" ")
                } else {
                    全部參數.join(" ")
                };
                Ok(format!("{} {}", style("✓").green(), 描述))
            }
            Err(err) => Ok(format!("{} {err}", style("✗").red())),
        },
        Err(err) => Ok(format!("{} {err}", style("✗").red())),
    }
}

fn 讀取可取消(prompt: &str, _主題: &ColorfulTheme) -> anyhow::Result<Option<String>> {
    let mut 取消主題 = ColorfulTheme::default();
    取消主題.success_prefix = style("↩".to_string()).for_stderr().cyan();
    let 輸入: String = Input::with_theme(&取消主題)
        .with_prompt(prompt)
        .allow_empty(true)
        .interact_text()?;
    let trimmed = 輸入.trim();
    let normalized = trimmed.trim_start_matches(|c| c == '/' || c == ':');
    if normalized.eq_ignore_ascii_case("cancel")
        || trimmed.eq_ignore_ascii_case("c")
        || trimmed.eq_ignore_ascii_case("q")
        || normalized.eq_ignore_ascii_case("q")
        || normalized.eq_ignore_ascii_case("c")
        || normalized == "q"
        || normalized == "取消"
        || normalized == "退出"
    {
        Ok(None)
    } else {
        Ok(Some(輸入))
    }
}

fn 非空或無(s: String) -> Option<String> {
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

fn 包含選項(args: &[String], key: &str) -> bool {
    args.iter().any(|a| a == key)
}

fn 橫線() -> String {
    let 終端寬度 = Term::stdout().size().1 as usize;
    "-".repeat(終端寬度 / 2)
}
