use anyhow::{anyhow, bail};
use rime::{
    rime_api_call, rime_module_call, rime_struct_new, RimeConfig, RimeLeversApi, RimeTraits,
};
use std::ffi::{CStr, CString};
use std::path::PathBuf;
#[derive(Default)]
pub struct 引擎啓動參數 {
    pub 用戶數據場地: PathBuf,
    pub 共享數據場地: Option<PathBuf>,
    pub 品名: Option<String>,
    pub 代號: Option<String>,
    pub 版本: Option<String>,
    pub 應用名: Option<String>,
    pub 最小日誌級別: Option<i32>,
    pub 日誌場地: Option<PathBuf>,
    pub 預構建固件場地: Option<PathBuf>,
    pub 緩存場地: Option<PathBuf>,
}
impl 引擎啓動參數 {
    pub fn 新建(用戶數據場地: PathBuf) -> Self {
        Self {
            用戶數據場地,
            ..Default::default()
        }
    }
}

pub fn 設置引擎啓動參數(參數: &引擎啓動參數) -> anyhow::Result<()> {
    log::debug!("設置引擎啓動參數. 用戶數據場地: {}", 參數.用戶數據場地.display());
    std::fs::create_dir_all(&參數.用戶數據場地)?;
    let 用戶數據場地〇 = CString::new(參數.用戶數據場地.to_str().ok_or(anyhow!("路徑編碼轉換錯誤"))?)?;
    let 共享數據場地〇 = if let Some(ref 場地) = 參數.共享數據場地 {
        std::fs::create_dir_all(場地)?;
        Some(CString::new(場地.to_str().ok_or(anyhow!("路徑編碼轉換錯誤"))?)?)
    } else {
        None
    };
    let 品名〇 = if let Some(ref 品名) = 參數.品名 {
        Some(CString::new(品名.as_str())?)
    } else {
        Some(CString::new(env!("CARGO_PKG_NAME"))?)
    };
    let 代號〇 = if let Some(ref 代號) = 參數.代號 {
        Some(CString::new(代號.as_str())?)
    } else {
        品名〇.clone()
    };
    let 版本〇 = if let Some(ref 版本) = 參數.版本 {
        Some(CString::new(版本.as_str())?)
    } else {
        Some(CString::new(env!("CARGO_PKG_VERSION"))?)
    };
    let 應用名〇 = 參數.應用名.as_ref().map(|s| CString::new(s.as_str())).transpose()?;
    let 日誌場地〇 = if let Some(ref p) = 參數.日誌場地 {
        std::fs::create_dir_all(p)?;
        Some(CString::new(p.to_str().ok_or(anyhow!("路徑編碼轉換錯誤"))?)?)
    } else {
        None
    };
    let 預構建固件場地〇 = if let Some(ref p) = 參數.預構建固件場地 {
        std::fs::create_dir_all(p)?;
        Some(CString::new(p.to_str().ok_or(anyhow!("路徑編碼轉換錯誤"))?)?)
    } else {
        None
    };
    let 緩存場地〇 = if let Some(ref p) = 參數.緩存場地 {
        std::fs::create_dir_all(p)?;
        Some(CString::new(p.to_str().ok_or(anyhow!("路徑編碼轉換錯誤"))?)?)
    } else {
        None
    };

    let mut 啓動參數: RimeTraits = rime_struct_new!();
    啓動參數.data_size = std::mem::size_of::<RimeTraits>() as std::ffi::c_int;
    啓動參數.shared_data_dir = 共享數據場地〇.as_ref().map_or(用戶數據場地〇.as_ptr(), |s| s.as_ptr());
    啓動參數.user_data_dir = 用戶數據場地〇.as_ptr();
    啓動參數.distribution_name = 品名〇.as_ref().map_or(std::ptr::null(), |s| s.as_ptr());
    啓動參數.distribution_code_name = 代號〇.as_ref().map_or(std::ptr::null(), |s| s.as_ptr());
    啓動參數.distribution_version = 版本〇.as_ref().map_or(std::ptr::null(), |s| s.as_ptr());
    if !參數.應用名.is_none() {
        啓動參數.app_name = 應用名〇.as_ref().map_or(std::ptr::null(), |s| s.as_ptr());
    }
    if !參數.最小日誌級別.is_none() {
        啓動參數.min_log_level = 參數.最小日誌級別.unwrap();
    }
    if !參數.日誌場地.is_none() {
        啓動參數.log_dir = 日誌場地〇.as_ref().map_or(std::ptr::null(), |s| s.as_ptr());
    }
    if !參數.預構建固件場地.is_none() {
        啓動參數.prebuilt_data_dir = 預構建固件場地〇.as_ref().map_or(std::ptr::null(), |s| s.as_ptr());
    }
    if !參數.緩存場地.is_none() {
        啓動參數.staging_dir = 緩存場地〇.as_ref().map_or(std::ptr::null(), |s| s.as_ptr());
    }
    rime_api_call!(setup, &mut 啓動參數);
    Ok(())
}

pub fn 製備輸入法固件() -> anyhow::Result<()> {
    log::debug!("製備輸入法固件");
    rime_api_call!(deployer_initialize, std::ptr::null_mut());
    rime_api_call!(deploy);
    rime_api_call!(finalize);
    Ok(())
}

pub fn 配置補丁(目標配置: &str, 紐: &str, 值: &str) -> anyhow::Result<()> {
    log::debug!("配置補丁: {目標配置}:/{紐} = {值}");

    let 目標配置〇 = CString::new(目標配置)?;
    let 紐〇 = CString::new(紐)?;
    let 值〇 = CString::new(值)?;

    let mut 值解析爲節點樹: RimeConfig = rime_struct_new!();
    if rime_api_call!(config_load_string, &mut 值解析爲節點樹, 值〇.as_ptr()) == 0 {
        bail!("無效的 YAML 值: {}", 值);
    }

    let levers_模塊名〇 = CString::new("levers")?;
    let levers = rime_api_call!(find_module, levers_模塊名〇.as_ptr());
    if levers.is_null() {
        bail!("沒有 levers 模塊");
    }

    let 配置工具名稱〇 = CString::new("rime-cli")?;
    let 自定義配置 = rime_module_call!(
        levers => RimeLeversApi,
        custom_settings_init,
        目標配置〇.as_ptr(),
        配置工具名稱〇.as_ptr()
    );

    // 可能已有自定義配置, 先加載
    rime_module_call!(levers => RimeLeversApi, load_settings, 自定義配置);
    // 生成補丁
    if rime_module_call!(
        levers => RimeLeversApi,
        customize_item,
        自定義配置,
        紐〇.as_ptr(),
        &mut 值解析爲節點樹
    ) != 0
    {
        rime_module_call!(levers => RimeLeversApi, save_settings, 自定義配置);
        log::info!("補丁打好了. {目標配置}:/{紐}");
    }

    rime_module_call!(levers => RimeLeversApi, custom_settings_destroy, 自定義配置);
    rime_api_call!(config_close, &mut 值解析爲節點樹);

    Ok(())
}

pub fn 加入輸入方案列表(衆輸入方案: &[String]) -> anyhow::Result<()> {
    log::debug!("加入輸入方案列表: {:#?}", 衆輸入方案);
    rime_api_call!(deployer_initialize, std::ptr::null_mut());

    let mut 自定義配置: RimeConfig = rime_struct_new!();
    let 默認配置的自定義〇 = CString::new("default.custom")?;
    rime_api_call!(
        user_config_open,
        默認配置的自定義〇.as_ptr(),
        &mut 自定義配置
    );
    let mut 既有方案 = vec![];
    let 方案列表〇 = CString::new("patch/schema_list")?;
    let 既有方案數 = rime_api_call!(config_list_size, &mut 自定義配置, 方案列表〇.as_ptr()) as u64;
    for i in 0..既有方案數 {
        let 列表項〇 = CString::new(format!("patch/schema_list/@{}/schema", i))?;
        let 方案 = rime_api_call!(config_get_cstring, &mut 自定義配置, 列表項〇.as_ptr());
        if !方案.is_null() {
            既有方案.push(unsafe { CStr::from_ptr(方案) }.to_str()?.to_owned());
        }
    }
    let 新增方案 = 衆輸入方案.iter().filter(|方案| !既有方案.contains(方案));
    let 新增列表項〇 = CString::new("patch/schema_list/@next/schema")?;
    for 方案 in 新增方案 {
        let 方案〇 = CString::new(方案.to_owned())?;
        rime_api_call!(
            config_set_string,
            &mut 自定義配置,
            新增列表項〇.as_ptr(),
            方案〇.as_ptr()
        );
    }
    rime_api_call!(config_close, &mut 自定義配置);

    rime_api_call!(finalize);
    Ok(())
}

pub fn 從方案列表中刪除(衆輸入方案: &[String]) -> anyhow::Result<()> {
    log::debug!("從方案列表中刪除: {:#?}", 衆輸入方案);
    // 從default.custom.yaml中刪除指定方案的配置項
    rime_api_call!(deployer_initialize, std::ptr::null_mut());
    let mut 自訂配置: RimeConfig = rime_struct_new!();
    let 默認配置的自訂〇 = CString::new("default.custom")?;
    rime_api_call!(
        user_config_open,
        默認配置的自訂〇.as_ptr(),
        &mut 自訂配置
    );
    // 取得已有方案列表
    let 方案列表〇 = CString::new("patch/schema_list")?;
    let 既有方案數 = rime_api_call!(config_list_size, &mut 自訂配置, 方案列表〇.as_ptr()) as u64;
    let mut 既有方案 = vec![];
    for i in 0..既有方案數 {
        let 列表項〇 = CString::new(format!("patch/schema_list/@{}/schema", i))?;
        let 方案 = rime_api_call!(config_get_cstring, &mut 自訂配置, 列表項〇.as_ptr());
        if !方案.is_null() {
            既有方案.push(unsafe { CStr::from_ptr(方案) }.to_str()? .to_owned());
        }
    }
    let 保留方案 = 既有方案.into_iter().filter(|方案| !衆輸入方案.contains(方案));

    rime_api_call!(config_create_list, &mut 自訂配置, 方案列表〇.as_ptr());
    for (i, 方案) in 保留方案.enumerate() {
        let 列表項〇 = CString::new(format!("patch/schema_list/@{}/schema", i))?;
        let 方案〇 = CString::new(方案.to_owned())?;
        rime_api_call!(
            config_set_string,
            &mut 自訂配置,
            列表項〇.as_ptr(),
            方案〇.as_ptr()
        );
    }
    rime_api_call!(config_close, &mut 自訂配置);
    rime_api_call!(finalize);
    Ok(())
}

pub fn 選擇輸入方案(方案: &str) -> anyhow::Result<()> {
    log::debug!("選擇輸入方案: {方案}");
    rime_api_call!(deployer_initialize, std::ptr::null_mut());

    let mut 用戶配置: RimeConfig = rime_struct_new!();
    let 用戶配置〇 = CString::new("user")?;
    rime_api_call!(user_config_open, 用戶配置〇.as_ptr(), &mut 用戶配置);
    let 用家之選〇 = CString::new("var/previously_selected_schema")?;
    let 方案〇 = CString::new(方案.to_owned())?;
    rime_api_call!(
        config_set_string,
        &mut 用戶配置,
        用家之選〇.as_ptr(),
        方案〇.as_ptr()
    );
    rime_api_call!(config_close, &mut 用戶配置);

    rime_api_call!(finalize);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use claims::assert_ok;
    use std::fs::{read_to_string, write};
    use std::sync::RwLock;

    fn 獲取公共測試場地() -> PathBuf {
        std::env::temp_dir().join("rime_levers_tests")
    }

    // rime::Deployer 是個單例, 同一時刻只能服務一片場地.
    // 公共場地中的測試可以並發執行, 持讀鎖. 專用場地的測試持寫鎖.
    static 佔用引擎機位: RwLock<()> = RwLock::new(());

    fn 預備(場地: &PathBuf) {
        if 場地.exists() {
            std::fs::remove_dir_all(場地).unwrap();
        }
        let 參數 = 引擎啓動參數::新建(場地.clone());
        assert_ok!(設置引擎啓動參數(&參數));
    }

    #[test]
    fn 測試配置補丁_全局配置() {
        let _佔 = 佔用引擎機位.write().unwrap_or_else(|e| e.into_inner());
        let 場地 = 獲取公共測試場地();
        預備(&場地);
        assert_ok!(配置補丁("default", "menu/page_size", "5"));

        let 結果文件 = 場地.join("default.custom.yaml");
        let 補丁文件內容 = assert_ok!(read_to_string(&結果文件));
        assert!(補丁文件內容.replace('\r', "").contains(
            r#"patch:
  "menu/page_size": 5"#
        ));
    }

    #[test]
    fn 測試配置補丁_輸入方案() {
        let _佔 = 佔用引擎機位.write().unwrap_or_else(|e| e.into_inner());
        let 場地 = 獲取公共測試場地();
        預備(&場地);
        assert_ok!(配置補丁("ohmyrime.schema", "menu/page_size", "9"));

        let 結果文件 = 場地.join("ohmyrime.custom.yaml");
        let 補丁文件內容 = assert_ok!(read_to_string(&結果文件));
        assert!(補丁文件內容.replace('\r', "").contains(
            r#"patch:
  "menu/page_size": 9"#
        ));
    }

    #[test]
    fn 測試配置補丁_列表值() {
        let _佔 = 佔用引擎機位.write().unwrap_or_else(|e| e.into_inner());
        let 場地 = 獲取公共測試場地();
        預備(&場地);
        assert_ok!(配置補丁(
            "patch_list",
            "starcraft/races",
            r#"[protoss, terran, zerg]"#
        ));

        let 結果文件 = 場地.join("patch_list.custom.yaml");
        let 補丁文件內容 = assert_ok!(read_to_string(&結果文件));
        assert!(補丁文件內容.replace('\r', "").contains(
            r#"patch:
  "starcraft/races":
    - protoss
    - terran
    - zerg"#
        ));
    }

    #[test]
    fn 測試配置補丁_字典值() {
        let _佔 = 佔用引擎機位.write().unwrap_or_else(|e| e.into_inner());
        let 場地 = 獲取公共測試場地();
        預備(&場地);
        assert_ok!(配置補丁(
            "patch_map",
            "starcraft/workers",
            r#"{protoss: probe, terran: scv, zerg: drone}"#
        ));

        let 結果文件 = 場地.join("patch_map.custom.yaml");
        let 補丁文件內容 = assert_ok!(read_to_string(&結果文件));
        assert!(補丁文件內容.replace('\r', "").contains(
            r#"patch:
  "starcraft/workers":
    protoss: probe
    terran: scv
    zerg: drone"#
        ));
    }

    #[test]
    fn 測試製備輸入法固件() {
        let _佔 = 佔用引擎機位.write().unwrap_or_else(|e| e.into_inner());
        let 專用測試場地 = std::env::temp_dir().join("rime_levers_tests_build");
        if 專用測試場地.exists() {
            println!("清理舊有測試場地: {}", 專用測試場地.display());
            assert_ok!(std::fs::remove_dir_all(&專用測試場地));
        }
        let 參數 = 引擎啓動參數::新建(專用測試場地.clone());
        println!("user data dir: {}", 參數.用戶數據場地.display());
        println!("shared data dir: {:?}", 參數.共享數據場地.as_ref().map(|p| p.display()));
        assert_ok!(設置引擎啓動參數(&參數));
        assert_ok!(write(
            專用測試場地.join("default.yaml"),
            r#"
schema_list:
  - schema: ohmyrime
"#,
        ));
        assert_ok!(write(
            專用測試場地.join("ohmyrime.schema.yaml"),
            r#"
schema:
  schema_id: ohmyrime
"#,
        ));

        assert_ok!(製備輸入法固件());

        assert!(專用測試場地.join("installation.yaml").exists());
        assert!(專用測試場地.join("user.yaml").exists());
        let 整備區 = 專用測試場地.join("build");
        let 默認配置文件 = 整備區.join("default.yaml");
        let 默認配置內容 = assert_ok!(read_to_string(&默認配置文件));
        assert!(默認配置內容.replace('\r', "").contains(
            r#"schema_list:
  - schema: ohmyrime"#
        ));
        let 輸入方案文件 = 整備區.join("ohmyrime.schema.yaml");
        let 輸入方案內容 = assert_ok!(read_to_string(&輸入方案文件));
        assert!(輸入方案內容.replace('\r', "").contains(
            r#"schema:
  schema_id: ohmyrime"#
        ));
    }

    #[test]
    fn 測試加入輸入方案列表() {
        let _佔 = 佔用引擎機位.write().unwrap_or_else(|e| e.into_inner());
        let 專用測試場地 = std::env::temp_dir().join("rime_levers_tests_add");
        if 專用測試場地.exists() {
            assert_ok!(std::fs::remove_dir_all(&專用測試場地));
        }
        let 參數 = 引擎啓動參數::新建(專用測試場地.clone());
        assert_ok!(設置引擎啓動參數(&參數));

        let 新增輸入方案 = vec!["protoss".to_owned(), "terran".to_owned()];
        assert_ok!(加入輸入方案列表(&新增輸入方案));

        let 自訂配置檔案 = 專用測試場地.join("default.custom.yaml");
        assert!(自訂配置檔案.exists());
        let 自訂配置內容 = assert_ok!(read_to_string(&自訂配置檔案));
        assert!(自訂配置內容.replace('\r', "").contains(
            r#"patch:
  schema_list:
    - {schema: protoss}
    - {schema: terran}"#
        ));

        let 新增輸入方案 = vec!["terran".to_owned(), "zerg".to_owned()];
        assert_ok!(加入輸入方案列表(&新增輸入方案));
        let 自訂配置內容 = assert_ok!(read_to_string(&自訂配置檔案));
        assert!(自訂配置內容.replace('\r', "").contains(
            r#"patch:
  schema_list:
    - {schema: protoss}
    - {schema: terran}
    - {schema: zerg}"#
        ));
    }

    #[test]
    fn 測試從方案列表中刪除() {
        let _佔 = 佔用引擎機位.write().unwrap_or_else(|e| e.into_inner());
        let 專用測試場地 = std::env::temp_dir().join("rime_levers_tests_remove");
        if 專用測試場地.exists() {
            assert_ok!(std::fs::remove_dir_all(&專用測試場地));
        }
        let 參數 = 引擎啓動參數::新建(專用測試場地.clone());
        assert_ok!(設置引擎啓動參數(&參數));
        let 初始輸入方案 = vec![
            "protoss".to_owned(),
            "terran".to_owned(),
            "zerg".to_owned(),
        ];
        assert_ok!(加入輸入方案列表(&初始輸入方案));
        let 自訂配置檔案 = 專用測試場地.join("default.custom.yaml");
        assert!(自訂配置檔案.exists());
        let 自訂配置內容 = assert_ok!(read_to_string(&自訂配置檔案));
        assert!(自訂配置內容.replace('\r', "").contains(
            r#"patch:
  schema_list:
    - {schema: protoss}
    - {schema: terran}
    - {schema: zerg}"#
        ));
        let 待刪除方案 = vec!["protoss".to_owned(), "terran".to_owned()];
        assert_ok!(從方案列表中刪除(&待刪除方案));
        // 重新檢查剩餘數量
        rime_api_call!(deployer_initialize, std::ptr::null_mut());
        let mut 自訂配置: RimeConfig = rime_struct_new!();
        let 默認配置的自訂〇 = CString::new("default.custom").unwrap();
        rime_api_call!(
            user_config_open,
            默認配置的自訂〇.as_ptr(),
            &mut 自訂配置
        );
        let 方案列表〇 = CString::new("patch/schema_list").unwrap();
        let 既有方案數 = rime_api_call!(config_list_size, &mut 自訂配置, 方案列表〇.as_ptr()) as u64;
        assert_eq!(既有方案數, 1);
        rime_api_call!(config_close, &mut 自訂配置);
        rime_api_call!(finalize);
        let 自訂配置內容 = assert_ok!(read_to_string(&自訂配置檔案));
        assert!(自訂配置內容.replace('\r', "").contains(
            r#"patch:
  schema_list:
    - {schema: zerg}"#
        ));
    }

    #[test]
    fn 測試選擇輸入方案() {
        let _佔 = 佔用引擎機位.write().unwrap_or_else(|e| e.into_inner());
        let 專用測試場地 = std::env::temp_dir().join("rime_levers_tests_select");
        if 專用測試場地.exists() {
            assert_ok!(std::fs::remove_dir_all(&專用測試場地));
        }
        let 參數 = 引擎啓動參數::新建(專用測試場地.clone());
        assert_ok!(設置引擎啓動參數(&參數));

        let grrrr_之選 = "protoss";
        assert_ok!(選擇輸入方案(grrrr_之選));

        let 用戶配置 = 專用測試場地.join("user.yaml");
        assert!(用戶配置.exists());
        let 用戶配置內容 = assert_ok!(read_to_string(&用戶配置));
        assert!(用戶配置內容.replace('\r', "").contains(
            r#"var:
  previously_selected_schema: protoss"#
        ));

        let boxer_之選 = "terran";
        assert_ok!(選擇輸入方案(boxer_之選));

        let 用戶配置內容 = assert_ok!(read_to_string(&用戶配置));
        assert!(用戶配置內容.replace('\r', "").contains(
            r#"var:
  previously_selected_schema: terran"#
        ));
    }
}
