use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::Result;

use config::Config;

use crate::config;

static RES_PATH: OnceLock<PathBuf> = OnceLock::new();
static APKS_PATH: OnceLock<PathBuf> = OnceLock::new();

static APP_PATH: OnceLock<PathBuf> = OnceLock::new();

static AAB_PATH: OnceLock<PathBuf> = OnceLock::new();
static BASE_ZIP_PATH: OnceLock<PathBuf> = OnceLock::new();
static BASE_DIR_PATH: OnceLock<PathBuf> = OnceLock::new();
static ROOT_PATH: OnceLock<PathBuf> = OnceLock::new();
static MANIFEST_PATH: OnceLock<PathBuf> = OnceLock::new();
static ASSETS_PATH: OnceLock<PathBuf> = OnceLock::new();
static NEW_ASSETS_PATH: OnceLock<PathBuf> = OnceLock::new();

static LIB_PATH: OnceLock<PathBuf> = OnceLock::new();
static NEW_LIB_PATH: OnceLock<PathBuf> = OnceLock::new();

static BASE_ROOT_PATH: OnceLock<PathBuf> = OnceLock::new();
static UNKNOWN_PATH: OnceLock<PathBuf> = OnceLock::new();
static NEW_UNKNOWN_PATH: OnceLock<PathBuf> = OnceLock::new();
static KOTLIN_PATH: OnceLock<PathBuf> = OnceLock::new();
static NEW_KOTLIN_PATH: OnceLock<PathBuf> = OnceLock::new();

static META_PATH: OnceLock<PathBuf> = OnceLock::new();
static NEW_META_PATH: OnceLock<PathBuf> = OnceLock::new();
static DEX_PATH: OnceLock<PathBuf> = OnceLock::new();

static BASE_APK_PATH: OnceLock<PathBuf> = OnceLock::new();
static RESOURCES_ZIP_PATH: OnceLock<PathBuf> = OnceLock::new();

static APK_BUILD_PATH: OnceLock<PathBuf> = OnceLock::new();
static APK_DIST_PATH: OnceLock<PathBuf> = OnceLock::new();
static APK_UN_SIGN_PATH: OnceLock<PathBuf> = OnceLock::new();
static APK_ZIPALIGN_PATH: OnceLock<PathBuf> = OnceLock::new();
static APK_SIGN_PATH: OnceLock<PathBuf> = OnceLock::new();

pub fn get_res_path(config: &Config) -> Result<&PathBuf> {
    let res = RES_PATH.get_or_try_init(|| {
        let res_path = Path::new(config.apk.apk_outdir.as_str()).join("res");
        Ok(res_path)
    });
    res
}

pub fn get_apks_path(config: &Config) -> Result<&PathBuf> {
    let apks = APKS_PATH.get_or_try_init(|| {
        let apks_path = Path::new(config.apk.apk_outdir.as_str()).join("app.apks");
        Ok(apks_path)
    });
    apks
}

pub fn get_aab_path<'a>(config: &'a Config, time: &'a str) -> Result<&'a PathBuf> {
    let aab = AAB_PATH.get_or_try_init(|| {
        let aab_path = Path::new(config.apk.apk_outdir.as_str()).join(format!("{}{}.aab", time, config.apk.app_name));
        Ok(aab_path)
    });
    aab
}

pub fn get_base_zip_path(config: &Config) -> Result<&PathBuf> {
    let base_zip = BASE_ZIP_PATH.get_or_try_init(|| {
        let base_zip_path = Path::new(config.apk.apk_outdir.as_str()).join("base.zip");
        Ok(base_zip_path
        )
    });
    base_zip
}

pub fn get_base_dir_path(config: &Config) -> Result<&PathBuf> {
    let base_dir = BASE_DIR_PATH.get_or_try_init(|| {
        let base_dir_path = Path::new(config.apk.apk_outdir.as_str()).join("base");

        Ok(base_dir_path)
    });
    base_dir
}

pub fn get_root_path(config: &Config) -> Result<&PathBuf> {
    let root = ROOT_PATH.get_or_try_init(|| {
        let root_path = Path::new(config.apk.apk_outdir.as_str()).to_path_buf();

        Ok(root_path)
    });
    root
}

pub fn get_manifest_path(config: &Config) -> Result<&PathBuf> {
    let manifest = MANIFEST_PATH.get_or_try_init(|| {
        let manifest_path = get_base_dir_path(config)?.join("manifest");

        Ok(manifest_path)
    });
    manifest
}

pub fn get_assets_path(config: &Config) -> Result<&PathBuf> {
    let assets = ASSETS_PATH.get_or_try_init(|| {
        let assets_path = get_root_path(config)?.join("assets");

        Ok(assets_path)
    });
    assets
}

pub fn get_new_assets_path(config: &Config) -> Result<&PathBuf> {
    let new_assets = NEW_ASSETS_PATH.get_or_try_init(|| {
        let new_assets_path = get_base_dir_path(config)?.join("assets");

        Ok(new_assets_path)
    });
    new_assets
}

pub fn get_lib_path(config: &Config) -> Result<&PathBuf> {
    let lib = LIB_PATH.get_or_try_init(|| {
        let lib_path = get_root_path(config)?.join("lib");

        Ok(lib_path)
    });
    lib
}

pub fn get_new_lib_path(config: &Config) -> Result<&PathBuf> {
    let new_lib = NEW_LIB_PATH.get_or_try_init(|| {
        let new_lib_path = get_base_dir_path(config)?.join("lib");

        Ok(new_lib_path)
    });
    new_lib
}

pub fn get_base_root_path(config: &Config) -> Result<&PathBuf> {
    let base_root = BASE_ROOT_PATH.get_or_try_init(|| {
        let base_root_path = get_base_dir_path(config)?.join("root");

        Ok(base_root_path)
    });
    base_root
}

pub fn get_unknown_path(config: &Config) -> Result<&PathBuf> {
    let unknown = UNKNOWN_PATH.get_or_try_init(|| {
        let unknown_path = get_root_path(config)?.join("unknown");

        Ok(unknown_path)
    });
    unknown
}

pub fn get_new_unknown_path(config: &Config) -> Result<&PathBuf> {
    let new_unknown = NEW_UNKNOWN_PATH.get_or_try_init(|| {
        let new_unknown_path = get_base_root_path(config)?.join("root").join("unknown");

        Ok(new_unknown_path)
    });
    new_unknown
}

pub fn get_kotlin_path(config: &Config) -> Result<&PathBuf> {
    let kotlin = KOTLIN_PATH.get_or_try_init(|| {
        let kotlin_path = get_root_path(config)?.join("kotlin");

        Ok(kotlin_path)
    });
    kotlin
}

pub fn get_new_kotlin_path(config: &Config) -> Result<&PathBuf> {
    let new_kotlin = NEW_KOTLIN_PATH.get_or_try_init(|| {
        let new_kotlin_path = get_base_root_path(config)?.join("kotlin");

        Ok(new_kotlin_path)
    });
    new_kotlin
}

pub fn get_meta_path(config: &Config) -> Result<&PathBuf> {
    let meta = META_PATH.get_or_try_init(|| {
        let meta_path = get_root_path(config)?.join("original").join("META-INF");

        Ok(meta_path)
    });
    meta
}

pub fn get_new_meta_path(config: &Config) -> Result<&PathBuf> {
    let new_meta = NEW_META_PATH.get_or_try_init(|| {
        let new_meta_path = get_base_root_path(config)?.join("root").join("META-INF");

        Ok(new_meta_path)
    });
    new_meta
}

pub fn get_dex_path(config: &Config) -> Result<&PathBuf> {
    let dex = DEX_PATH.get_or_try_init(|| {
        let dex_path = get_base_dir_path(config)?.join("dex");

        Ok(dex_path)
    });
    dex
}

pub fn get_base_apk_path(config: &Config) -> Result<&PathBuf> {
    let base_apk = BASE_APK_PATH.get_or_try_init(|| {
        let base_apk_path = get_root_path(config)?.join("base.apk");

        Ok(base_apk_path)
    });
    base_apk
}

pub fn get_resources_zip_path(config: &Config) -> Result<&PathBuf> {
    let resources = RESOURCES_ZIP_PATH.get_or_try_init(|| {
        let resources_path = get_root_path(config)?.join("resources.zip");

        Ok(resources_path)
    });
    resources
}

pub fn get_app_path(config: &Config) -> Result<&PathBuf> {
    let app = APP_PATH.get_or_try_init(|| {
        let app_path = Path::new(config.build_apk.app_path.as_str()).to_path_buf();

        Ok(app_path)
    });
    app
}

pub fn get_apk_build_path(config: &Config) -> Result<&PathBuf> {
    let build = APK_BUILD_PATH.get_or_try_init(|| {
        let build_path = get_app_path(config)?.join("build");

        Ok(build_path)
    });
    build
}

pub fn get_apk_dist_path(config: &Config) -> Result<&PathBuf> {
    let dist = APK_DIST_PATH.get_or_try_init(|| {
        let dist_path = get_app_path(config)?.join("dist");

        Ok(dist_path)
    });
    dist
}

pub fn get_apk_un_sign_path<'a>(config: &'a Config, time: &'a str) -> Result<&'a PathBuf> {
    let un_sign = APK_UN_SIGN_PATH.get_or_try_init(|| {
        let un_sign_path = get_apk_dist_path(config)?.join(format!("{}_{}-unsign.apk", time, config.apk.app_name));

        Ok(un_sign_path)
    });
    un_sign
}

pub fn get_apk_zipalign_path<'a>(config: &'a Config, time: &'a str) -> Result<&'a PathBuf> {
    let zipalign = APK_ZIPALIGN_PATH.get_or_try_init(|| {
        let zipalign_path = get_apk_dist_path(config)?.join(format!("{}_{}-zip.apk", time, config.apk.app_name));

        Ok(zipalign_path)
    });
    zipalign
}

pub fn get_apk_sign_path<'a>(config: &'a Config, time: &'a str) -> Result<&'a PathBuf> {
    let sign = APK_SIGN_PATH.get_or_try_init(|| {
        let sign_path = get_apk_dist_path(config)?.join(format!("{}_{}-sign.apk", time, config.apk.app_name));

        Ok(sign_path)
    });
    sign
}
