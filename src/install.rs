use crate::recipe::配方名片;
use std::{path::PathBuf, fs};
use anyhow::Context;

pub fn 安裝配方(配方: &配方名片, 用戶目錄: &PathBuf) -> anyhow::Result<()> {
    let 配方包 = crate::package::配方包 {
        配方: 配方.clone(),
        倉庫域名: None,
    };
    let 配方目錄 = 配方包.本地路徑();
    fs::create_dir_all(&用戶目錄)
        .with_context(|| format!("創建用戶目錄失敗: {:?}", 用戶目錄))?;
    遞歸複製目錄(&配方目錄, &用戶目錄)
        .with_context(|| format!("安裝配方 {:?} 到 {:?}", 配方目錄, 用戶目錄))?;
    log::debug!("配方 {:?}/{:?} 安裝完成", 配方.方家, 配方.名字);
    Ok(())
}

fn 遞歸複製目錄(源: &PathBuf, 目標: &PathBuf) -> anyhow::Result<()> {
    log::debug!("複製目錄: {:?} 到 {:?}", 源, 目標);
    for 項目 in fs::read_dir(源)
        .with_context(|| format!("讀取目錄失敗: {:?}", 源))?
    {
        // 忽略.git .github README.md等文件,忽略大小寫
        if let Ok(入口) = &項目 {
            let 名字 = 入口.file_name();
            let 名字 = 名字.to_string_lossy().to_lowercase();
            if 名字 == ".git"
                || 名字 == ".github"
                || 名字 == "readme.md"
                || 名字 == "license"
                || 名字 == "license.txt"
                || 名字 == "authors"
                || 名字 == ".github"
            {
                continue;
            }
            if 名字 == "default.custom.yaml" {
                println!("發現自定義配置文件: {:?},可能會影響全局設定， 是否覆蓋? (y/yes 覆蓋, 默認其他 跳過)", 入口.path());
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                let input = input.trim().to_lowercase();
                if !(input == "y" || input == "yes") {
                    println!("跳過覆蓋自定義配置文件: {:?}", 入口.path());
                    continue;
                }
            }
        }
        let 項目 = 項目.with_context(|| "讀取目錄項失敗")?;
        let 文件類型 = 項目.file_type().with_context(|| "讀取文件類型失敗")?;
        let 目標路徑 = 目標.join(項目.file_name());

        if 文件類型.is_dir() {
            fs::create_dir_all(&目標路徑)
                .with_context(|| format!("創建目錄失敗: {:?}", 目標路徑))?;
            遞歸複製目錄(&項目.path(), &目標路徑)?;
        } else if 文件類型.is_file() {
            log::debug!("複製文件: {:?} 到 {:?}", 項目.path(), 目標路徑);
            fs::copy(項目.path(), &目標路徑)
                .with_context(|| format!("複製文件失敗: {:?} 到 {:?}", 項目.path(), 目標路徑))?;
        }
    }
    Ok(())
}
