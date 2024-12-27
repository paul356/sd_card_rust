use esp_idf_sys::{sdmmc_host_t, esp_vfs_fat_sdmmc_mount_config_t, esp_vfs_fat_sdmmc_mount, esp_vfs_fat_sdcard_unmount, sdmmc_card_t, sdmmc_host_init, ESP_OK, sdmmc_host_set_bus_width, sdmmc_host_get_slot_width, sdmmc_host_set_bus_ddr_mode, sdmmc_host_set_card_clk, sdmmc_host_set_cclk_always_on, sdmmc_host_do_transaction, sdmmc_host_deinit, sdmmc_host_io_int_enable, sdmmc_host_io_int_wait, sdmmc_host_get_real_freq, sdmmc_host_set_input_delay, sdmmc_host_get_dma_info, sdmmc_host_t__bindgen_ty_1};
use std::ffi::CString;
use std::os::raw::c_int;
use std::os::raw::c_uint;
use std::os::raw::c_uchar;
use std::os::raw::c_void;
use std::io::Write;
use std::io::Read;

#[repr(C)]
#[derive(Debug)]
struct sdmmc_slot_config_t {
    clk: c_int,
    cmd: c_int,
    d0: c_int,
    d1: c_int,
    d2: c_int,
    d3: c_int,
    d4: c_int,
    d5: c_int,
    d6: c_int,
    d7: c_int,
    cd: c_int,
    wp: c_int,
    width: c_uchar,
    flags: c_uint,
}

const SDMMC_SLOT_FLAG_INTERNAL_PULLUP: c_uint = 1 << 0;
const SDMMC_HOST_FLAG_1BIT: c_uint = 1 << 0;
const SDMMC_HOST_FLAG_4BIT: c_uint = 1 << 1;
const SDMMC_HOST_FLAG_8BIT: c_uint = 1 << 2;
const SDMMC_HOST_FLAG_DDR: c_uint  = 1 << 4;
const SDMMC_HOST_SLOT_1: i32 = 1;
const SDMMC_FREQ_DEFAULT: i32 = 20000;
const SDMMC_DELAY_PHASE_0: u32 = 0;

impl Default for sdmmc_slot_config_t {
    fn default() -> Self {
        Self {
            clk: 7,
            cmd: 6,
            d0: 15,
            d1: 16,
            d2: 4,
            d3: 5,
            d4: -1,
            d5: -1,
            d6: -1,
            d7: -1,
            cd: 17,
            wp: -1,
            width: 4,
            flags: SDMMC_SLOT_FLAG_INTERNAL_PULLUP,
        }
    }
}

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello, world!");

    let sdmmc_mount_config = esp_vfs_fat_sdmmc_mount_config_t {
        format_if_mount_failed: false,
        max_files: 4,
        allocation_unit_size: 16 * 1024,
        disk_status_check_enable: false,
        use_one_fat: false,
    };

    let mount_point = CString::new("/sdcard").unwrap();

    let sd_host = sdmmc_host_t {
        flags: SDMMC_HOST_FLAG_1BIT | SDMMC_HOST_FLAG_4BIT | SDMMC_HOST_FLAG_8BIT | SDMMC_HOST_FLAG_DDR,
        slot: SDMMC_HOST_SLOT_1,
        max_freq_khz: SDMMC_FREQ_DEFAULT,
        io_voltage: 3.3,
        init: Some(sdmmc_host_init),
        set_bus_width: Some(sdmmc_host_set_bus_width),
        get_bus_width: Some(sdmmc_host_get_slot_width),
        set_bus_ddr_mode: Some(sdmmc_host_set_bus_ddr_mode),
        set_card_clk: Some(sdmmc_host_set_card_clk),
        set_cclk_always_on: Some(sdmmc_host_set_cclk_always_on),
        do_transaction: Some(sdmmc_host_do_transaction),
        __bindgen_anon_1: sdmmc_host_t__bindgen_ty_1 {
            deinit: Some(sdmmc_host_deinit)
        },
        io_int_enable: Some(sdmmc_host_io_int_enable),
        io_int_wait: Some(sdmmc_host_io_int_wait),
        command_timeout_ms: 0,
        get_real_freq: Some(sdmmc_host_get_real_freq),
        input_delay_phase: SDMMC_DELAY_PHASE_0,
        set_input_delay: Some(sdmmc_host_set_input_delay),
        dma_aligned_buffer: std::ptr::null_mut(),
        pwr_ctrl_handle: std::ptr::null_mut(),
        get_dma_info: Some(sdmmc_host_get_dma_info),
    };

    let slot_config = sdmmc_slot_config_t::default();

    let mut card_handle: *mut sdmmc_card_t = std::ptr::null_mut();

    let ret = unsafe {
        esp_vfs_fat_sdmmc_mount(
            mount_point.as_ptr(),
            &sd_host,
            &slot_config as *const sdmmc_slot_config_t as *const c_void,
            &sdmmc_mount_config,
            &mut card_handle,
        )
    };

    match ret {
        ESP_OK => log::info!("SD Card mounted"),
        _ => {
            log::error!("Failed to mount SD Card");
            return;
        }
    }

    let file_res = std::fs::File::create_new("/sdcard/test.txt");
    let mut file = match file_res {
        Ok(mut file) => {
            file.write_all(b"Hello, world!").unwrap();

            std::fs::File::open("/sdcard/test.txt").unwrap()
        }
        Err(_) => {
            std::fs::File::open("/sdcard/test.txt").unwrap()
        }
    };

    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();

    log::info!("File content: {}", content);

    drop(file);

    unsafe {
        esp_vfs_fat_sdcard_unmount(mount_point.as_ptr(), card_handle);
    }
}
