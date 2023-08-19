## 功能
1. 自动化将`apk`转为`aab`文件.
2. 自动化实现打包`smali`,签名,安装.


## 用法
#### 配置文件(config.toml )
```
[sign]
#"exec command: java -jar {} build-apks --bundle {} --output {} --ks {} --ks-pass pass:{} --ks-key-alias {} --key-pass pass:{}", config.jar.bundletool_path, aab_path.to_string_lossy(), apks_path.to_string_lossy(), config.sign.keystore, config.sign.keystore_pass, config.sign.keystore_key_alias, config.sign.keystore_key_pass
keystore = "your_keystore_path/yourkeystore.jks"
keystore_pass= "your keystore password"
keystore_key_alias="your keystore alias"
keystore_key_pass="your key password"

[apk]
#"exec command: java -jar {} d {} -s -o {}", config.jar.apktool_path, config.apk.apk_path, config.apk.apk_outdir
apk_path="your_apk_path/yourapk.apk"
apk_outdir="your output dir"
#"exec command: aapt2 link --proto-format -o {} -I {} --min-sdk-version {} --target-sdk-version {} --version-code {} --version-name {} --manifest {} -R {} --auto-add-overlay",base_apk_path.to_string_lossy(), config.jar.android_jar_path, config.apk.min_sdk_version, config.apk.target_sdk_version, config.apk.version_code, config.apk.version_name,manifest_path.to_string_lossy(),res_zip_path.to_string_lossy()
min_sdk_version="21"
target_sdk_version="31"
version_code="101"
version_name="1.0.1"
#"{}_{}-sign.apk", time, config.apk.app_name
app_name="your app name"

[jar]
#"exec command: java -jar {} d {} -s -o {}", config.jar.apktool_path, config.apk.apk_path, config.apk.apk_outdir
apktool_path="your_apktool_path/apktool.jar"
#"exec command: aapt2 link --proto-format -o {} -I {} --min-sdk-version {} --target-sdk-version {} --version-code {} --version-name {} --manifest {} -R {} --auto-add-overlay",base_apk_path.to_string_lossy(), config.jar.android_jar_path, config.apk.min_sdk_version, config.apk.target_sdk_version, config.apk.version_code, config.apk.version_name,manifest_path.to_string_lossy(),res_zip_path.to_string_lossy()
bundletool_path="your_bundletool_path/bundletool-all-1.15.2.jar"
android_jar_path="your_sdkpath/sdk/platforms/android-31/android.jar"

[config]
#"exec command: adb install -r {}", apk_sign_path.to_string_lossy().to_string()
install = true
#"exec command: adb shell am start -n {}", config.config.main_activity
launch = true
main_activity = "your_package/your_launcher_activity"
#"exec command: java -jar {} build-bundle --modules {} --output {} --config={}", config.jar.bundletool_path, zip_path.to_string_lossy(), aab_path.to_string_lossy(), config.config.bundletool_config_path)
bundletool_config_path = "bundletool config file path"

[build_apk]
#"exec command: java -jar {} b {} -o {}", config.jar.apktool_path, config.build_apk.app_path, apk_unsign_path.to_string_lossy().to_string();
app_path="your smali dir"


```

#### apk转aab命令
```
./abtool_cli -c config.toml -a aab
```
`config.toml`为配置文件. `aab`为指定将`apk`转为`aab`流程.
主要流程如下:
```
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
 ```

#### 编译apk
```
./abtool_cli -c config.toml -a apk
```
将`smali`工程编译成`apk`并签名.
主要流程如下:
```
    apktool_build(&config, time)?;
    zipalign(&config, time)?;
    let apk_name = apksigner(&config, time)?;
    if config.config.install {
        install_apk(&config, time)?;
        if config.config.launch {
            launch_app(&config)?;
        }
    }
  ```

## 源码
1. 编译
```
 cargo build -p abtool_cli --release 
 ```
2. upx压缩(可选)
   在`target/release`下
```
 upx --best --lzma abtool_cli  
 ```

压缩后`linux`平台二进制文件大小在`880k`左右.

3. 从源码运行
```
cargo run -p abtool_cli -- -c your_config_file_path/config.toml -a aab
```

## 跨平台
目前在`linux`设备上完美运行,理论上已经适配了`Windows`与`mac OS`系统, 但未能拿到相关设备进行测试.



`github`项目地址:`https://github.com/tangxuesong6/abtool`.