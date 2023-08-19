use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Sign {
    pub(crate) keystore: String,
    pub(crate) keystore_pass: String,
    pub(crate) keystore_key_alias: String,
    pub(crate) keystore_key_pass: String,
}

#[derive(Debug, Deserialize)]
pub struct Apk {
    pub(crate) apk_path: String,
    pub(crate) apk_outdir: String,
    pub(crate) min_sdk_version: String,
    pub(crate) target_sdk_version: String,
    pub(crate) version_code: String,
    pub(crate) version_name: String,
    pub(crate) app_name: String,
}

#[derive(Debug, Deserialize)]
pub struct Jar {
    pub(crate) apktool_path: String,
    pub(crate) bundletool_path: String,
    pub(crate) android_jar_path: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub(crate) sign: Sign,
    pub(crate) apk: Apk,
    pub(crate) jar: Jar,
    pub(crate) config: ConfigInfo,
    pub(crate) build_apk: BuildApk,
}

#[derive(Debug, Deserialize)]
pub struct ConfigInfo {
    pub(crate) install: bool,
    pub(crate) launch: bool,
    pub(crate) main_activity: String,
    pub(crate) bundletool_config_path: String,
}

#[derive(Debug, Deserialize)]
pub struct BuildApk {
    pub(crate) app_path: String,
}