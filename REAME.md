这个例子需要配合ESP<sub>IDF</sub> 5.3才能工作，似乎ESP<sub>IDF</sub> 5.2的SDCARD相关代码还不够完善，所以你需要使用安装好的ESP-IDF，需要在Cargo.toml中加入如下的配置，指定使用环境变量关联的ESP-IDF 5.3。

    [package.metadata.esp-idf-sys]
    esp_idf_tools_install_dir = "fromenv"
    esp_idf_sdkconfig = "sdkconfig"
    esp_idf_sdkconfig_defaults = ["sdkconfig.defaults", "sdkconfig.defaults.ble"]  

如何配置环境可以参考[准备开发环境](https://paul356.github.io/2024/11/11/rust-on-esp-series_1.html), 有什么问题欢迎提Issue交流。

