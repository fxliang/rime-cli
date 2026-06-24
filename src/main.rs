use std::path::PathBuf;

use rime_cli::app::{執行命令, 場地參數, 子命令};
use rime_cli::client::{設置共享數據目錄覆蓋, 設置用戶目錄覆蓋};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Rime 配方管理器")]
struct Cli {
    #[structopt(flatten)]
    場地: 場地參數,
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

    #[structopt(subcommand)]
    command: Option<子命令>,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let cli = Cli::from_args();
    log::debug!("參數: {:?}", cli.command);

    if let Some(ref user_dir) = cli.user_data_dir {
        設置用戶目錄覆蓋(Some(user_dir.clone()));
    }
    if let Some(ref shared_dir) = cli.shared_data_dir {
        設置共享數據目錄覆蓋(Some(shared_dir.clone()));
    }

    match cli.command {
        #[cfg(feature = "tui")]
        Some(子命令::Tui) => rime_cli::tui::進入tui(&cli.場地),
        Some(命令行參數) => 執行命令(命令行參數, &cli.場地, false),
        None => {
            #[cfg(feature = "tui")]
            {
                rime_cli::tui::進入tui(&cli.場地)
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
