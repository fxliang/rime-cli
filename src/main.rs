use std::{path::PathBuf};
use structopt::StructOpt;

mod download;
mod install;
mod package;
mod recipe;
mod rime_levers;
mod get_rime;

use download::{下載參數, 下載配方包};
use install::安裝配方;
use recipe::配方名片;
use rime_levers::{
    加入輸入方案列表, 製備輸入法固件, 設置引擎啓動參數, 選擇輸入方案, 配置補丁
};
// rppi_parser crate is a workspace dependency

#[derive(Debug, StructOpt)]
#[structopt(about = "Rime 配方管理器")]
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
    /// 安裝配方
    Install {
        /// 要安裝的配方
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

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let 命令行參數 = 子命令::from_args();
    log::debug!("參數: {:?}", 命令行參數);

    執行命令(命令行參數)
}

fn 執行命令(命令行參數: 子命令) -> anyhow::Result<()> {
    match 命令行參數 {
        子命令::Add { schemata } => {
            let 還不知道怎麼傳過來 = PathBuf::from(".");
            設置引擎啓動參數(&還不知道怎麼傳過來)?;
            加入輸入方案列表(&schemata)?;
        }
        子命令::Build => {
            let 還不知道怎麼傳過來 = PathBuf::from(".");
            設置引擎啓動參數(&還不知道怎麼傳過來)?;
            製備輸入法固件()?;
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
                安裝配方(配方)?;
            }
        }
        子命令::Patch { config, key, value } => {
            let 還不知道怎麼傳過來 = PathBuf::from(".");
            設置引擎啓動參數(&還不知道怎麼傳過來)?;
            配置補丁(&config, &key, &value)?;
        }
        子命令::Select { schema } => {
            選擇輸入方案(&schema)?;
        }
        子命令::Get { tag, 下載參數 } => {
            let 目標版本 = tag.unwrap_or("".to_string());
            get_rime::更新引擎庫(&目標版本, &下載參數)?;
        }
        #[cfg(feature = "tui")]
        子命令::Tui => tui::進入tui()?,
        _ => todo!("還沒做呢"),
    }

    Ok(())
}

#[cfg(feature = "tui")]
mod tui {
    use super::*;
    use crate::download::{下載參數, 同步rppi索引};
    use dialoguer::{theme::ColorfulTheme, Input, Select};
    use dialoguer::console::{style, Term};
    use rppi_parser::{CatalogNode, Recipe, load_catalog};
    use std::{collections::HashSet, fs, path::PathBuf};

    #[derive(Copy, Clone)]
    enum 配方操作 {
        Download,
        Install,
    }

    #[derive(Copy, Clone)]
    enum 配方選擇來源 {
        手動,
        Rppi,
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
                        if let (Some(鍵), Some(值)) = (分隔.next(),分隔.next()) {
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

    pub fn 進入tui() -> anyhow::Result<()> {
        let 主題 = ColorfulTheme::default();
        let 終端 = Term::stdout();
        let mut 配置 = 讀取tui配置()?;
        let mut proxy = 配置.proxy.clone();
        let mut host = 配置.host.clone();
        let mut rppi索引: Option<PathBuf> = None;
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
            let sel = Select::with_theme(&主題)
                .items(&選項)
                .default(0)
                .interact_on_opt(&終端)?;
            let 應退出 = match sel {
                Some(0) => {
                    if let Some(msg) = 處理下載或安裝(配方操作::Download, &主題, &終端, host.as_deref(), proxy.as_deref(), &mut rppi索引)? {
                        狀態 = Some(msg);
                    }
                    false
                }
                Some(1) => {
                    if let Some(msg) = 處理下載或安裝(配方操作::Install, &主題, &終端, host.as_deref(), proxy.as_deref(), &mut rppi索引)? {
                        狀態 = Some(msg);
                    }
                    false
                }
                Some(2) => {
                    let tag = match 讀取可取消("版本標籤 (留空使用最新)", &主題)? {
                        Some(t) => t,
                        None => continue 'tui,
                    };
                    let mut args = vec!["get".to_string()];
                    if !tag.trim().is_empty() {
                        args.push(tag);
                    }
                    狀態 = Some(執行tui命令參數(args, host.as_deref(), proxy.as_deref())?);
                    false
                }
                Some(3) => {
                    let Some(schema) = 讀取可取消("選擇的輸入方案", &主題)? else {
                        continue 'tui;
                    };
                    if !schema.trim().is_empty() {
                        狀態 = Some(執行tui命令參數(
                            vec!["select".to_string(), schema],
                            host.as_deref(),
                            proxy.as_deref(),
                        )?);
                    }
                    false
                }
                Some(4) => {
                    let Some(輸入) = 讀取可取消("要加入的輸入方案 (空格分隔)", &主題)? else {
                        continue 'tui;
                    };
                    let 方案 = 輸入
                        .split_whitespace()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>();
                    if !方案.is_empty() {
                        let mut args = vec!["add".to_string()];
                        args.extend(方案);
                        狀態 = Some(執行tui命令參數(args, host.as_deref(), proxy.as_deref())?);
                    }
                    false
                }
                Some(5) => {
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
                            host.as_deref(),
                            proxy.as_deref(),
                        )?);
                    }
                    false
                }
                Some(6) => {
                    狀態 = Some(執行tui命令參數(vec!["build".to_string()], host.as_deref(), proxy.as_deref())?);
                    false
                }
                Some(7) => {
                    let 輸入: String = Input::with_theme(&主題)
                        .with_prompt("Proxy (留空清除)")
                        .allow_empty(true)
                        .interact_text()?;
                    proxy = 非空或無(輸入);
                    配置.proxy = proxy.clone();
                    保存tui配置(&配置)?;
                    false
                }
                Some(8) => {
                    let 輸入: String = Input::with_theme(&主題)
                        .with_prompt("Host (留空清除)")
                        .allow_empty(true)
                        .interact_text()?;
                    host = 非空或無(輸入);
                    配置.host = host.clone();
                    保存tui配置(&配置)?;
                    false
                }
                None => true,
                _ => true,
            };
            if 應退出 {
                break;
            }
        }

        保存tui配置(&配置)?;
        Ok(())
    }

    fn 處理下載或安裝(
        操作: 配方操作,
        主題: &ColorfulTheme,
        終端: &Term,
        host: Option<&str>,
        proxy: Option<&str>,
        rppi索引: &mut Option<PathBuf>,
    ) -> anyhow::Result<Option<String>> {
        let Some(來源) = 選擇配方來源(主題, 終端)? else {
            return Ok(None);
        };

        match 來源 {
            配方選擇來源::手動 => {
                let 提示 = match 操作 {
                    配方操作::Download => "要下載的配方 (空格分隔)",
                    配方操作::Install => "要安裝的配方 (空格分隔)",
                };
                let Some(輸入) = 讀取可取消(提示, 主題)? else {
                    return Ok(None);
                };
                let 配方 = 輸入
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>();
                if 配方.is_empty() {
                    return Ok(None);
                }
                let mut 訊息 = Vec::new();
                for p in 配方 {
                    let mut args = vec![match 操作 {
                        配方操作::Download => "download".to_string(),
                        配方操作::Install => "install".to_string(),
                    }];
                    let prompt = if args[0] == "install" {
                        format!("安裝配方 {}", p)
                    } else {
                        format!("下載配方 {}", p)
                    };
                    println!("{}", style(prompt).blue());
                    args.push(p);
                    訊息.push(執行tui命令參數(args, host, proxy)?);
                }
                Ok(Some(訊息.join("\n")))
            }
            配方選擇來源::Rppi => {
                if let Some(配方列表) = 從rppi選擇配方(主題, 終端, host, proxy, rppi索引)? {
                    let mut 訊息 = Vec::new();
                    for 配方 in 配方列表 {
                        let mut args = vec![match 操作 {
                            配方操作::Download => "download".to_string(),
                            配方操作::Install => "install".to_string(),
                        }];
                        let prompt = if args[0] == "install" {
                            format!("安裝配方 {}", 配方)
                        } else {
                            format!("下載配方 {}", 配方)
                        };
                        println!("{}", style(prompt).blue());
                        args.push(配方);
                        訊息.push(執行tui命令參數(args, host, proxy)?);
                    }
                    Ok(Some(訊息.join("\n")))
                } else {
                    Ok(None)
                }
            }
        }
    }

    fn 選擇配方來源(主題: &ColorfulTheme, 終端: &Term) -> anyhow::Result<Option<配方選擇來源>> {
        let 選項 = vec![
            "直接輸入配方".to_string(),
            "瀏覽 rime/rppi".to_string(),
            "返回".to_string(),
        ];
        let sel = Select::with_theme(主題)
            .items(&選項)
            .default(0)
            .interact_on_opt(終端)?;
        match sel {
            Some(0) => Ok(Some(配方選擇來源::手動)),
            Some(1) => Ok(Some(配方選擇來源::Rppi)),
            None => Ok(None),
            _ => Ok(None),
        }
    }

    fn 從rppi選擇配方(
        主題: &ColorfulTheme,
        終端: &Term,
        host: Option<&str>,
        proxy: Option<&str>,
        rppi索引: &mut Option<PathBuf>,
    ) -> anyhow::Result<Option<Vec<String>>> {
        let 參數 = 下載參數::new(
            host.map(|h| h.to_string()),
            proxy.map(|p| p.to_string()),
            None,
        );
        let rppi目錄 = if let Some(已拉取) = rppi索引 {
            已拉取.clone()
        } else {
            let 拉取路徑 = 同步rppi索引(&參數)?;
            *rppi索引 = Some(拉取路徑.clone());
            拉取路徑
        };
        let catalog = load_catalog(&rppi目錄)?;
        Ok(選擇rppi配方(&catalog, 主題, 終端)?.map(|r| rppi配方列表(&r)))
    }

    enum Rppi菜單條目 {
        分類 { key: String },
        配方(Recipe),
        返回,
        取消,
    }

    fn 選擇rppi配方(
        catalog: &CatalogNode,
        主題: &ColorfulTheme,
        終端: &Term,
    ) -> anyhow::Result<Option<Recipe>> {
        let mut 當前 = catalog;
        let mut 堆疊: Vec<&CatalogNode> = Vec::new();

        loop {
            let mut 條目列表: Vec<Rppi菜單條目> = Vec::new();
            let mut 顯示文本: Vec<String> = Vec::new();

            if let Some(parent) = &當前.parent {
                for cat in &parent.categories {
                    let label = format!("{} ({})", cat.name, cat.key);
                    條目列表.push(Rppi菜單條目::分類 { key: cat.key.clone() });
                    顯示文本.push(label);
                }
            }

            if let Some(child) = &當前.child {
                for recipe in &child.recipes {
                    let label = format!("{} ({})", recipe.name, recipe.repo);
                    條目列表.push(Rppi菜單條目::配方(recipe.clone()));
                    顯示文本.push(label);
                }
            }

            if !堆疊.is_empty() {
                條目列表.push(Rppi菜單條目::返回);
                顯示文本.push("返回上級".to_string());
            }

            條目列表.push(Rppi菜單條目::取消);
            顯示文本.push("取消".to_string());

            if 顯示文本.is_empty() {
                return Ok(None);
            }

            let sel = Select::with_theme(主題)
                .items(&顯示文本)
                .default(0)
                .interact_on_opt(終端)?;

            match sel {
                None => return Ok(None), // Esc -> go back
                Some(idx) => match 條目列表.get(idx).unwrap_or(&Rppi菜單條目::取消) {
                    Rppi菜單條目::分類 { key } => {
                        if let Some(next) = 當前.children.get(key) {
                            堆疊.push(當前);
                            當前 = next;
                        }
                    }
                    Rppi菜單條目::配方(recipe) => return Ok(Some(recipe.clone())),
                    Rppi菜單條目::返回 => {
                        if let Some(prev) = 堆疊.pop() {
                            當前 = prev;
                        }
                    }
                    Rppi菜單條目::取消 => return Ok(None),
                },
            }
        }
    }

    fn rppi配方列表(recipe: &Recipe) -> Vec<String> {
        let mut 已見 = HashSet::new();
        let mut 列表 = Vec::new();

        let 主配方 = rppi配方串(recipe);
        已見.insert(主配方.clone());
        列表.push(主配方);

        if let Some(deps) = &recipe.dependencies {
            for d in deps {
                if 已見.insert(d.clone()) {
                    列表.push(d.clone());
                }
            }
        }
        if let Some(rdeps) = &recipe.reverse_dependencies {
            for d in rdeps {
                if 已見.insert(d.clone()) {
                    列表.push(d.clone());
                }
            }
        }

        列表
    }

    fn rppi配方串(recipe: &Recipe) -> String {
        let mut spec = recipe.repo.clone();
        if let Some(branch) = &recipe.branch {
            spec.push('@');
            spec.push_str(branch);
        }
        spec
    }

    fn 執行tui命令參數(
        args: Vec<String>,
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
            Ok(cmd) => match 執行命令(cmd) {
                Ok(()) => {
                    let 描述 = if 全部參數.len() > 1 {
                        全部參數[1..].join(" ")
                    } else {
                        全部參數.join(" ")
                    };
                    Ok(format!("{} {}", style("✓").green(), 描述))
                }
                Err(err) => Ok(format!("{} {err}", style("✗").red())),
            }
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
}
