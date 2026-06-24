use std::path::PathBuf;

use rime_cli::app::{執行命令, 子命令};
use rime_cli::client::{啓動上下文, 啓動上下文覆蓋};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    about = "Rime 配方管理器",
    help_message = "顯示幫助信息",
    version_message = "顯示版本信息",
    template = "{bin} {version}
{about}

用法:
    {usage}

選項:
{unified}

子命令:
{subcommands}"
)]
struct Cli {
    /// 指定用户数据目录，覆盖平台默认值
    #[structopt(long = "user-data-dir", short = "u", parse(from_os_str), global(true))]
    user_data_dir: Option<PathBuf>,

    /// 指定共享数据目录，覆盖平台默认值
    #[structopt(
        long = "shared-data-dir",
        short = "s",
        parse(from_os_str),
        global(true)
    )]
    shared_data_dir: Option<PathBuf>,

    /// 指定日誌目錄，覆蓋默認位置
    #[structopt(long = "log-dir", parse(from_os_str), global(true))]
    log_dir: Option<PathBuf>,

    /// 指定 librime app_name
    #[structopt(long = "app-name", global(true))]
    app_name: Option<String>,

    /// 指定發行品名 distribution_name
    #[structopt(long = "distribution-name", global(true))]
    distribution_name: Option<String>,

    /// 指定發行代號 distribution_code_name
    #[structopt(long = "distribution-code-name", global(true))]
    distribution_code_name: Option<String>,

    /// 指定發行版本 distribution_version
    #[structopt(long = "distribution-version", global(true))]
    distribution_version: Option<String>,

    /// 指定 librime 最小日誌級別
    #[structopt(long = "min-log-level", global(true))]
    min_log_level: Option<i32>,

    /// 指定預構建數據目錄 prebuilt_data_dir
    #[structopt(long = "prebuilt-data-dir", parse(from_os_str), global(true))]
    prebuilt_data_dir: Option<PathBuf>,

    /// 指定緩存/整備目錄 staging_dir
    #[structopt(long = "staging-dir", parse(from_os_str), global(true))]
    staging_dir: Option<PathBuf>,

    #[structopt(subcommand)]
    command: Option<子命令>,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let cli = Cli::from_args();
    log::debug!("參數: {:?}", cli.command);

    let 上下文 = 啓動上下文::解析(啓動上下文覆蓋 {
        用戶數據目錄: cli.user_data_dir,
        共享數據目錄: cli.shared_data_dir,
        日誌目錄: cli.log_dir,
        應用名: cli.app_name,
        發行品名: cli.distribution_name,
        發行代號: cli.distribution_code_name,
        發行版本: cli.distribution_version,
        最小日誌級別: cli.min_log_level,
        預構建數據目錄: cli.prebuilt_data_dir,
        緩存目錄: cli.staging_dir,
    })?;

    match cli.command {
        #[cfg(feature = "tui")]
        Some(子命令::Tui) => rime_cli::tui::進入tui(上下文),
        Some(命令行參數) => 執行命令(&上下文, 命令行參數, false),
        None => {
            #[cfg(feature = "tui")]
            {
                rime_cli::tui::進入tui(上下文)
            }
            #[cfg(not(feature = "tui"))]
            {
                Cli::clap().print_help()?;
                println!();
                Ok(())
            }
        }
    }
}
