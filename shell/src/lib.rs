#![feature(once_cell_try)]

use std::{fs, io};
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{anyhow, Result};
use md5::digest::FixedOutput;
use md5::Md5;
use tracing::{debug, info, trace};
use walkdir::{DirEntry, WalkDir};
use zip::write::FileOptions;
use zip::ZipArchive;

use config::Config;

mod config;
mod file_path;

pub fn build_apk(config: String, time: &str) -> Result<String> {
    let config = read_config(config)?;
    apktool_rm_cache(&config)?;
    apktool_build(&config, time)?;
    zipalign(&config, time)?;
    let apk_name = apksigner(&config, time)?;
    if config.config.install {
        install_apk(&config, time)?;
        if config.config.launch {
            launch_app(&config)?;
        }
    }

    Ok(apk_name)
}

pub fn build_aab(config: String, time: &str) -> Result<String> {
    // let filtered_env : HashMap<String, String> =
    //     env::vars().filter(|&(ref k, _)|
    //         k == "TERM" || k == "TZ" || k == "LANG" || k == "PATH"
    //     ).collect();

    let config = read_config(config)?;

    let outdir = Path::new(config.apk.apk_outdir.as_str());

    if !outdir.exists() {
        decode_apk(&config)?;
    }

    compile_resources(&config)?;
    link_resources(&config)?;
    unzip_apk(&config)?;
    copy_resources(&config)?;
    zip_resources(&config)?;
    compile_app_bundle(&config, time)?;
    let aab_name = sign_app_bundle(&config, time)?;

    if config.config.install {
        build_apks(&config, time)?;
        install_apks(&config)?;
        if config.config.launch {
            launch_app(&config)?;
        }
    }

    Ok(aab_name)
}

fn read_config(config: String) -> Result<Config> {
    debug!("read config");
    let cfg_file = fs::read_to_string(config)?;
    debug!(cfg_file);
    let config: Config = toml::from_str(cfg_file.as_str())?;
    debug!("read config success");
    Ok(config)
}

fn install_apks(config: &Config) -> Result<()> {
    debug!("install apks");
    let apks_path = file_path::get_apks_path(config)?;
    info!("exec command: java -jar {} install-apks --apks {}", config.jar.bundletool_path, apks_path.to_string_lossy());

    let child = Command::new("java")
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .arg("-jar")
        .arg(config.jar.bundletool_path.as_str())
        .arg("install-apks")
        .arg("--apks")
        .arg(apks_path.to_string_lossy().to_string())
        .output()?;
    if child.status.success() {
        debug!("install apks success");
        Ok(())
    } else {
        Err(anyhow!("install apks failed"))
    }
}

fn build_apks(config: &Config, time: &str) -> Result<()> {
    debug!("build apks");
    let apks_path = file_path::get_apks_path(config)?;
    let aab_path = file_path::get_aab_path(config, time)?;
    info!("exec command: java -jar {} build-apks --bundle {} --output {} --ks {} --ks-pass pass:{} --ks-key-alias {} --key-pass pass:{}", config.jar.bundletool_path, aab_path.to_string_lossy(), apks_path.to_string_lossy(), config.sign.keystore, config.sign.keystore_pass, config.sign.keystore_key_alias, config.sign.keystore_key_pass);

    if apks_path.exists() {
        fs::remove_file(apks_path)?;
    }
    let child = Command::new("java")
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .arg("-jar")
        .arg(config.jar.bundletool_path.as_str())
        .arg("build-apks")
        .arg("--bundle")
        .arg(aab_path.to_string_lossy().to_string())
        .arg("--output")
        .arg(apks_path.to_string_lossy().to_string())
        .arg("--ks")
        .arg(config.sign.keystore.as_str())
        .arg("--ks-pass")
        .arg(format!("pass:{}", config.sign.keystore_pass).as_str())
        .arg("--ks-key-alias")
        .arg(config.sign.keystore_key_alias.as_str())
        .arg(format!("--key-pass=pass:{}", config.sign.keystore_key_pass).as_str())
        .output()?;

    if child.status.success() {
        debug!("build apks success");
        Ok(())
    } else {
        Err(anyhow!("build apks failed"))
    }
}

fn sign_app_bundle(config: &Config, time: &str) -> Result<String> {
    debug!("sign app bundle");
    let aab_path = file_path::get_aab_path(config, time)?;
    info!("exec command: jarsigner -digestalg SHA1 -sigalg SHA1withRSA -keystore {} -storepass {} -keypass {} {} {}", config.sign.keystore, config.sign.keystore_pass, config.sign.keystore_key_pass, aab_path.to_string_lossy(), config.sign.keystore_key_alias);
    let child = Command::new("jarsigner")
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .arg("-digestalg")
        .arg("SHA1")
        .arg("-sigalg")
        .arg("SHA1withRSA")
        .arg("-keystore")
        .arg(config.sign.keystore.as_str())
        .arg("-storepass")
        .arg(config.sign.keystore_pass.as_str())
        .arg("-keypass")
        .arg(config.sign.keystore_key_pass.as_str())
        .arg(aab_path.to_string_lossy().to_string())
        .arg(config.sign.keystore_key_alias.as_str())
        .output()?;
    if child.status.success() {
        debug!("sign app bundle success");
        Ok(aab_path.to_string_lossy().to_string())
    } else {
        Err(anyhow!("sign app bundle failed"))
    }
}

fn compile_app_bundle(config: &Config, time: &str) -> Result<()> {
    debug!("compile app bundle");
    let aab_path = file_path::get_aab_path(config, time)?;
    let zip_path = file_path::get_base_zip_path(config)?;


    if aab_path.exists() {
        fs::remove_file(aab_path)?;
    }
    let child: std::process::Output;
    if config.config.bundletool_config_path.is_empty() {
        info!("exec command: java -jar {} build-bundle --modules {} --output {}", config.jar.bundletool_path, zip_path.to_string_lossy(), aab_path.to_string_lossy());
        child = Command::new("java")
            .stderr(Stdio::inherit())
            .stdout(Stdio::inherit())
            .arg("-jar")
            .arg(config.jar.bundletool_path.as_str())
            .arg("build-bundle")
            .arg("--modules")
            .arg(zip_path.to_string_lossy().to_string())
            .arg("--output")
            .arg(aab_path.to_string_lossy().to_string())
            .output()?;
    } else {
        info!("exec command: java -jar {} build-bundle --modules {} --output {} --config={}", config.jar.bundletool_path, zip_path.to_string_lossy(), aab_path.to_string_lossy(), config.config.bundletool_config_path);
        child = Command::new("java")
            .stderr(Stdio::inherit())
            .stdout(Stdio::inherit())
            .arg("-jar")
            .arg(config.jar.bundletool_path.as_str())
            .arg("build-bundle")
            .arg("--modules")
            .arg(zip_path.to_string_lossy().to_string())
            .arg("--output")
            .arg(aab_path.to_string_lossy().to_string())
            .arg(format!("--config={}", config.config.bundletool_config_path).as_str())
            .output()?;
    }


    if child.status.success() {
        debug!("compile app bundle success");
        Ok(())
    } else {
        Err(anyhow!("compile app bundle failed"))
    }
}

fn zip_resources(config: &Config) -> Result<()> {
    debug!("zip resources");

    let zip_path = file_path::get_base_zip_path(config)?;

    if zip_path.exists() {
        fs::remove_file(zip_path.as_path())?;
    }
    let file = File::create(zip_path.as_path())?;

    let base_path = file_path::get_base_dir_path(config)?;

    let walkdir = WalkDir::new(Path::new(base_path.as_path()));
    let it = walkdir.into_iter();

    zip_dir(&mut it.filter_map(|e| e.ok()), base_path.to_string_lossy().to_string().as_str(), file, zip::CompressionMethod::Deflated)?;
    debug!("zip resources success");
    Ok(())
}

fn copy_resources(config: &Config) -> Result<()> {
    debug!("copy resources");
    let root_path = file_path::get_root_path(config)?;
    let base_path = file_path::get_base_dir_path(config)?;
    //创建 base/manifest
    let manifest_path = file_path::get_manifest_path(config)?;
    let _manifest = fs::create_dir_all(&manifest_path)?;
    cut_file(base_path.join("AndroidManifest.xml").to_string_lossy().to_string().as_str(), manifest_path.join("AndroidManifest.xml").to_string_lossy().to_string().as_str())?;
    //拷贝assets
    let assets_path = file_path::get_assets_path(config)?;

    if assets_path.exists() {
        let new_assets_path = file_path::get_new_assets_path(config)?;

        let _assets = fs::create_dir_all(&new_assets_path)?;
        copy_dir(assets_path.as_path(), new_assets_path.as_path())?;
    }

    //拷贝lib
    let lib_path = file_path::get_lib_path(config)?;

    if lib_path.exists() {
        let new_lib_path = file_path::get_new_lib_path(config)?;

        let _lib = fs::create_dir_all(&new_lib_path)?;

        copy_dir(lib_path.as_path(), new_lib_path.as_path())?;
    }

    let base_root = file_path::get_base_root_path(config)?;
    let _root = fs::create_dir_all(&base_root)?;

    //拷贝unknown
    let unknown_path = file_path::get_unknown_path(config)?;
    if unknown_path.exists() {
        let base_root_unknown = file_path::get_new_unknown_path(config)?;
        let _unknown = fs::create_dir_all(base_root_unknown.as_path())?;

        copy_dir(unknown_path.as_path(), base_root_unknown.as_path())?;
    }
    //拷贝kotlin
    let kotlin_path = file_path::get_kotlin_path(config)?;
    if kotlin_path.exists() {
        let new_kotlin_path = file_path::get_new_kotlin_path(config)?;
        let _kotlin = fs::create_dir_all(new_kotlin_path.as_path())?;
        copy_dir(kotlin_path.as_path(), new_kotlin_path.as_path())?;
    }
    //拷贝META-INF

    let meta_path = file_path::get_meta_path(config)?;

    if meta_path.exists() {
        let new_meta_path = file_path::get_new_meta_path(config)?;
        let _meta = fs::create_dir_all(new_meta_path.as_path())?;
        copy_dir(meta_path.as_path(), new_meta_path.as_path())?;
        for entry in fs::read_dir(new_meta_path.as_path())? {
            let entry = entry?;

            if !entry.file_type()?.is_dir() {
                if entry.file_name().to_string_lossy().contains(".RSA") || entry.file_name().to_string_lossy().contains(".MF") || entry.file_name().to_string_lossy().contains(".SF") {
                    fs::remove_file(entry.path())?;
                }
            }
        }
    }

    //dex 文件夹
    let dex_path = file_path::get_dex_path(config)?;
    let _dex = fs::create_dir_all(&dex_path)?;
    //拷贝classes.dex
    for entry in fs::read_dir(root_path)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            if entry.file_name().to_string_lossy().contains(".dex") {
                fs::copy(entry.path(), dex_path.join(entry.file_name().to_string_lossy().to_string()))?;
            }
        }
    }
    debug!("copy resources success");
    Ok(())
}

fn unzip_apk(config: &Config) -> Result<()> {
    debug!("unzip apk");
    let base_apk_path = file_path::get_base_apk_path(config)?;
    let base_path = file_path::get_base_dir_path(config)?;

    unzip(base_apk_path.to_string_lossy().to_string().as_str(), base_path.to_string_lossy().to_string().as_str())?;
    debug!("unzip apk success");
    Ok(())
}

fn link_resources(config: &Config) -> Result<()> {
    debug!("link resources");
    let res_zip_path = file_path::get_resources_zip_path(config)?;
    let base_apk_path = file_path::get_base_apk_path(config)?;
    let manifest_path = Path::new(config.apk.apk_outdir.as_str()).join("AndroidManifest.xml");
    info!("exec command: aapt2 link --proto-format -o {} -I {} --min-sdk-version {} --target-sdk-version {} --version-code {} --version-name {} --manifest {} -R {} --auto-add-overlay",base_apk_path.to_string_lossy(), config.jar.android_jar_path, config.apk.min_sdk_version, config.apk.target_sdk_version, config.apk.version_code, config.apk.version_name,manifest_path.to_string_lossy(),res_zip_path.to_string_lossy());
    if base_apk_path.exists() {
        fs::remove_file(base_apk_path.as_path())?;
    }

    let child = Command::new("aapt2")
        .arg("link")
        .arg("--proto-format")
        .arg("-o")
        .arg(base_apk_path.to_string_lossy().to_string().as_str())
        .arg("-I")
        .arg(format!("{}", config.jar.android_jar_path).as_str())
        .arg("--min-sdk-version")
        .arg(format!("{}", config.apk.min_sdk_version).as_str())
        .arg("--target-sdk-version")
        .arg(format!("{}", config.apk.target_sdk_version).as_str())
        .arg("--version-code")
        .arg(format!("{}", config.apk.version_code).as_str())
        .arg("--version-name")
        .arg(format!("{}", config.apk.version_name).as_str())
        .arg("--manifest")
        .arg(manifest_path.to_string_lossy().to_string().as_str())
        .arg("-R")
        .arg(res_zip_path.to_string_lossy().to_string().as_str())
        .arg("--auto-add-overlay")
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .output()
        .expect("failed to execute process");
    if child.status.success() {
        debug!("link resources success");
        Ok(())
    } else {
        Err(anyhow!("link resources failed"))
    }
}

fn compile_resources(config: &Config) -> Result<()> {
    debug!("compile resources");
    let res_path = file_path::get_res_path(config)?;
    let res_zip_path = file_path::get_resources_zip_path(config)?;

    info!("exec command: aapt2 compile --dir {} -o {}", res_path.to_string_lossy().to_string(), res_zip_path.to_string_lossy().to_string());

    if res_zip_path.exists() {
        fs::remove_file(res_zip_path.as_path())?;
    }
    let child = Command::new("aapt2")
        .arg("compile")
        .arg("--dir")
        .arg(res_path.to_string_lossy().to_string())
        .arg("-o")
        .arg(res_zip_path.to_string_lossy().to_string())
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .output()?;
    // .expect("failed to execute process");
    if child.status.success() {
        debug!("compile resources success");
        Ok(())
    } else {
        Err(anyhow!("compile resources failed"))
    }
}

//decode apk
fn decode_apk(config: &Config) -> Result<()> {
    debug!("decode apk");
    info!("exec command: java -jar {} d {} -s -o {}", config.jar.apktool_path, config.apk.apk_path, config.apk.apk_outdir);
    let child = Command::new("java")
        .args(&["-jar", format!("{}", config.jar.apktool_path).as_str(), "d", format!("{}", config.apk.apk_path).as_str(), "-s", "-o", format!("{}", config.apk.apk_outdir).as_str()])
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .output()?;
    if child.status.success() {
        debug!("decode apk success");
        Ok(())
    } else {
        Err(anyhow!("decode apk failed"))
    }
}


fn zip_dir<T>(
    it: &mut dyn Iterator<Item=DirEntry>,
    prefix: &str,
    writer: T,
    method: zip::CompressionMethod,
) -> Result<()>
    where
        T: Write + Seek,
{
    debug!("zip dir {}", prefix);
    let mut zip = zip::ZipWriter::new(writer);
    let options = FileOptions::default()
        .compression_method(method)
        .unix_permissions(0o755);

    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(Path::new(prefix))?;

        // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if path.is_file() {
            zip.start_file(path_to_string(name), options)?;
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            // Only if not root! Avoids path spec / warning
            // and mapname conversion failed error on unzip
            zip.add_directory(path_to_string(name), options)?;
        }
    }
    zip.finish()?;
    debug!("zip dir {} success", prefix);
    Ok(())
}

fn path_to_string(path: &std::path::Path) -> String {
    let mut path_str = String::new();
    for component in path.components() {
        if let std::path::Component::Normal(os_str) = component {
            if !path_str.is_empty() {
                path_str.push('/');
            }
            path_str.push_str(&os_str.to_string_lossy());
        }
    }
    path_str
}
#[cfg(target_os = "windows")]
fn path_to_string(path: &std::path::Path) -> String {
    let mut path_str = String::new();
    for component in path.components() {
        if let std::path::Component::Normal(os_str) = component {
            if !path_str.is_empty() {
                path_str.push('\\');
            }
            path_str.push_str(&os_str.to_string_lossy());
        }
    }
    path_str
}

fn unzip(zip_file: &str, dest_folder: &str) -> Result<()> {
    trace!("unzip {} to {}", zip_file, dest_folder);
    // 打开zip文件
    let mut zip_archive = ZipArchive::new(File::open(zip_file)?)?;

    // 遍历zip中的每一个文件
    for i in 0..zip_archive.len() {
        let mut file = zip_archive.by_index(i)?;

        // 构造解压后的文件路径
        let outpath = match file.enclosed_name() {
            Some(path) => Path::new(dest_folder).join(path),
            None => continue,
        };

        // 创建父文件夹
        if let Some(p) = outpath.parent() {
            if !p.exists() {
                std::fs::create_dir_all(p)?;
            }
        }

        // 解压文件到指定路径
        let mut outfile = File::create(&outpath)?;
        std::io::copy(&mut file, &mut outfile)?;

        // 设置文件权限
        // #[cfg(unix)]
        // {
        //     use std::os::unix::fs::PermissionsExt;
        //     if let Some(mode) = file.unix_mode() {
        //         std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode))?;
        //     }
        // }
    }
    trace!("unzip {} to {} success", zip_file, dest_folder);

    Ok(())
}


fn cut_file(from: &str, to: &str) -> Result<()> {
    debug!("cut file from {} to {}", from, to);
    fs::copy(from, to)?;

    let from_path = Path::new(from);
    let to_path = Path::new(to);

    let from_md5 = md5_from_file(from_path)?;
    let to_md5 = md5_from_file(to_path)?;

    if from_md5.finalize_fixed().to_vec() != to_md5.finalize_fixed().to_vec() {
        return Err(anyhow!("md5 not match"));
    }

    fs::remove_file(from)?;
    debug!("cut file from {} to {} success", from, to);
    Ok(())
}

fn md5_from_file(path: &Path) -> io::Result<Md5> {
    let mut file = fs::File::open(path)?;
    let mut hash = Md5::default();
    io::copy(&mut file, &mut hash)?;
    Ok(hash)
}

fn copy_dir(src: &Path, dst: &Path) -> Result<()> {
    trace!("copy dir from {:?} to {:?}", src, dst);
    // 检查源文件夹存在性
    if !src.exists() {
        return Err(anyhow!("src dir {:?} not exists", src));
    }

    let src_meta = src.metadata()?;

    // 如果源不是文件夹,作为单个文件复制
    if !src_meta.is_dir() {
        fs::copy(src, dst)?;
        return Ok(());
    }

    // 检查目标文件夹
    let dst_exists = dst.exists();
    let dst_meta = if dst_exists { dst.metadata()? } else { fs::metadata(".")? };

    if !dst_meta.is_dir() {
        return Err(anyhow!("dst dir {:?} not exists", dst));
    }

    // 目标文件夹为空,直接复制
    if dst_exists && dst_meta.len() == 0 {
        // fs::create_dir_all(dst)?;
        copy_dir_content(src, dst)?;

        // 目标文件夹存在,仅复制内容
    } else if dst_exists {
        copy_dir_content(src, dst)?;

        // 目标不存在,创建后复制
    } else {
        fs::create_dir(dst)?;
        copy_dir_content(src, dst)?;
    }
    trace!("copy dir from {:?} to {:?} success", src, dst);
    Ok(())
}

fn get_total_size(src: &Path) -> io::Result<u64> {
    let metadata = fs::metadata(src)?;

    if metadata.is_file() {
        return Ok(metadata.len());
    }

    let mut size = 0;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        size += get_total_size(&entry.path())?;
    }

    Ok(size)
}

fn copy_dir_content(src: &Path, dst: &Path) -> Result<()> {
    // debug!("copy dir content from {:?} to {:?}", src, dst);
    let _total_size = get_total_size(src)?;
    // let mut copied = 0;

    for entry in fs::read_dir(src)? {
        let entry = entry?;

        if entry.file_type()?.is_dir() {
            let _size = copy_dir(&entry.path(), &dst.join(entry.file_name()))?;
            // copied += size;
        } else {
            let _size = entry.metadata()?.len();
            fs::copy(&entry.path(), &dst.join(entry.file_name()))?;
            // copied += size;
        }

        // println!("{:.2}% ({}/{} bytes)", copied as f64 / total_size as f64 * 100.0, copied, total_size);
    }
    // debug!("copy dir content from {:?} to {:?} success", src, dst);
    Ok(())
}

fn launch_app(config: &Config) -> Result<()> {
    debug!("launch app");
    info!("exec command: adb shell am start -n {}", config.config.main_activity);
    let child = Command::new("adb")
        .arg("shell")
        .arg("am")
        .arg("start")
        .arg("-n")
        .arg(format!("{}", config.config.main_activity))
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .output()?;
    if child.status.success() {
        debug!("launch app success");
        Ok(())
    } else {
        Err(anyhow!("launch app failed"))
    }
}

fn apktool_rm_cache(config: &Config) -> Result<()> {
    debug!("apktool rm cache");
    let cache_path = file_path::get_apk_build_path(config)?;
    if cache_path.exists() {
        fs::remove_dir_all(&cache_path)?;
    }

    let dist_path = file_path::get_apk_dist_path(config)?;
    if dist_path.exists() {
        fs::remove_dir_all(&dist_path)?;
    }
    debug!("apktool rm cache success");
    Ok(())
}


fn apktool_build(config: &Config, time: &str) -> Result<()> {
    debug!("apktool build");
    let apk_unsign_path = file_path::get_apk_un_sign_path(config, time)?;
    info!("exec command: java -jar {} b {} -o {}", config.jar.apktool_path, config.build_apk.app_path, apk_unsign_path.to_string_lossy().to_string());
    let child = Command::new("java")
        .arg("-jar")
        .arg(config.jar.apktool_path.as_str())
        .arg("b")
        .arg(config.build_apk.app_path.as_str())
        .arg("-o")
        .arg(apk_unsign_path.to_string_lossy().to_string())
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .output()?;
    if child.status.success() {
        debug!("apktool build success");
        Ok(())
    } else {
        Err(anyhow!("apktool build failed"))
    }
}

fn zipalign(config: &Config, time: &str) -> Result<()> {
    debug!("zipalign");
    let apk_unsign_path = file_path::get_apk_un_sign_path(config, time)?;
    let apk_zipalign_path = file_path::get_apk_zipalign_path(config, time)?;
    info!("exec command: zipalign -v -p 4 {} {}", apk_unsign_path.to_string_lossy().to_string(), apk_zipalign_path.to_string_lossy().to_string());
    let child = Command::new("zipalign")
        .arg("-v")
        .arg("-p")
        .arg("4")
        .arg(apk_unsign_path.to_string_lossy().to_string())
        .arg(apk_zipalign_path.to_string_lossy().to_string())
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .output()?;
    if child.status.success() {
        debug!("zipalign success");
        Ok(())
    } else {
        Err(anyhow!("zipalign failed"))
    }
}

fn apksigner(config: &Config, time: &str) -> Result<String> {
    debug!("apksigner");
    let apk_zipalign_path = file_path::get_apk_zipalign_path(config, time)?;

    let apk_sign_path = file_path::get_apk_sign_path(config, time)?;

    info!("exec command: apksigner sign --ks {} --ks-pass pass:{} --out {} {}", config.sign.keystore, config.sign.keystore_pass, apk_sign_path.to_string_lossy().to_string(), apk_zipalign_path.to_string_lossy().to_string());
    let child = Command::new("apksigner")
        .arg("sign")
        .arg("--ks")
        .arg(config.sign.keystore.as_str())
        .arg("--ks-pass")
        .arg(format!("pass:{}", config.sign.keystore_pass))
        .arg("--out")
        .arg(apk_sign_path.to_string_lossy().to_string())
        .arg(apk_zipalign_path.to_string_lossy().to_string())
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .output()?;
    if child.status.success() {
        debug!("apksigner success");
        Ok(apk_sign_path.to_string_lossy().to_string())
    } else {
        Err(anyhow!("apksigner failed"))
    }
}

fn install_apk(config: &Config, time: &str) -> Result<()> {
    debug!("install apk");
    let apk_sign_path = file_path::get_apk_sign_path(config, time)?;
    info!("exec command: adb install -r {}", apk_sign_path.to_string_lossy().to_string());
    let child = Command::new("adb")
        .arg("install")
        .arg("-r")
        .arg(apk_sign_path.to_string_lossy().to_string())
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .output()?;
    if child.status.success() {
        debug!("install apk success");
        Ok(())
    } else {
        Err(anyhow!("install apk failed"))
    }
}





