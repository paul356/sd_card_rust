use std::ffi::CString;
use std::io::Read;
use std::io::Write;
use std::os::raw::c_uint;
use std::os::raw::c_void;

use display_interface_spi::SPIInterface;
use esp_idf_svc::hal::gpio;
use esp_idf_svc::hal::peripherals;
use esp_idf_svc::hal::spi;
use esp_idf_svc::hal::delay;
use esp_idf_svc::hal::units::FromValueType;
use esp_idf_svc::sys::{
    esp_vfs_fat_sdcard_unmount, esp_vfs_fat_sdmmc_mount, esp_vfs_fat_sdmmc_mount_config_t,
    sdmmc_card_t, sdmmc_host_deinit, sdmmc_host_do_transaction, sdmmc_host_get_dma_info,
    sdmmc_host_get_real_freq, sdmmc_host_get_slot_width, sdmmc_host_init, sdmmc_host_io_int_enable,
    sdmmc_host_io_int_wait, sdmmc_host_set_bus_ddr_mode, sdmmc_host_set_bus_width,
    sdmmc_host_set_card_clk, sdmmc_host_set_cclk_always_on, sdmmc_host_set_input_delay,
    sdmmc_host_t, sdmmc_host_t__bindgen_ty_1, sdmmc_slot_config_t,
    sdmmc_slot_config_t__bindgen_ty_1, sdmmc_slot_config_t__bindgen_ty_2, ESP_OK, EspError
};
use mipidsi;
use embedded_graphics::pixelcolor::RgbColor;
use embedded_graphics::primitives;
use embedded_graphics::prelude::*;

const SDMMC_SLOT_FLAG_INTERNAL_PULLUP: c_uint = 1 << 0;
const SDMMC_HOST_FLAG_1BIT: c_uint = 1 << 0;
const SDMMC_HOST_FLAG_4BIT: c_uint = 1 << 1;
const SDMMC_HOST_FLAG_8BIT: c_uint = 1 << 2;
const SDMMC_HOST_FLAG_DDR: c_uint = 1 << 4;
const SDMMC_HOST_SLOT_1: i32 = 1;
const SDMMC_FREQ_DEFAULT: i32 = 20000;
const SDMMC_DELAY_PHASE_0: u32 = 0;

fn get_slot_config() -> sdmmc_slot_config_t {
    sdmmc_slot_config_t {
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
        __bindgen_anon_1: sdmmc_slot_config_t__bindgen_ty_1 { cd: 17 },
        __bindgen_anon_2: sdmmc_slot_config_t__bindgen_ty_2 { wp: -1 },
        width: 4,
        flags: SDMMC_SLOT_FLAG_INTERNAL_PULLUP,
    }
}

fn test_display() -> Result<(), EspError> {
    log::info!("test display");

    let peripherals = peripherals::Peripherals::take().unwrap();

    let blk = peripherals.pins.gpio14;
    let dc = peripherals.pins.gpio13;
    let scl = peripherals.pins.gpio12;
    let sda = peripherals.pins.gpio11;
    let cs = peripherals.pins.gpio10;
    let rst = peripherals.pins.gpio9;

    let mut backlight = gpio::PinDriver::output(blk)?;
    backlight.set_high()?;

    let spi_drv = spi::SpiDeviceDriver::new_single(
        peripherals.spi2,
        scl,
        sda,
        Option::<gpio::Gpio13>::None,
        Some(cs),
        &spi::SpiDriverConfig::new().dma(spi::Dma::Disabled),
        &spi::SpiConfig::new().baudrate(26u32.MHz().into()),
    )?;

    let spi_intf = SPIInterface::new(spi_drv, gpio::PinDriver::output(dc)?);

    let mut display = mipidsi::Builder::new(mipidsi::models::ST7789, spi_intf)
        .reset_pin(gpio::PinDriver::output(rst)?)
        .color_order(mipidsi::options::ColorOrder::Rgb)
        .invert_colors(mipidsi::options::ColorInversion::Inverted)
        .display_size(240, 300) // display area is 240*280, but there is a -20 pixels offset
        .init(&mut delay::Ets).map_err(|e| {
            log::error!("Error initializing display: {:?}", e);
            EspError::from(-1).unwrap()
        })?;

    display.clear(RgbColor::BLUE)
        .map_err(|e| {
            log::error!("Error clear background: {:?}", e);
            EspError::from(-1).unwrap()
        })?;

    let style = primitives::PrimitiveStyleBuilder::new()
        .fill_color(RgbColor::WHITE)
        .stroke_color(RgbColor::RED)
        .stroke_width(1)
        .build();

    // It seems there is an voffset of -20 pixels
    let _ = primitives::Rectangle::new(Point::new(0, 20),
                                       Size::new(240, 280))
        .into_styled(style)
        .draw(&mut display)
        .map_err(|e| {
            log::error!("Error drawing rectangle: {:?}", e);
            EspError::from(-1).unwrap()
        })?;

    Ok(())
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
        flags: SDMMC_HOST_FLAG_1BIT
            | SDMMC_HOST_FLAG_4BIT
            | SDMMC_HOST_FLAG_8BIT
            | SDMMC_HOST_FLAG_DDR,
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
            deinit: Some(sdmmc_host_deinit),
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

    let slot_config = get_slot_config();

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
        Err(_) => std::fs::File::open("/sdcard/test.txt").unwrap(),
    };

    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();

    log::info!("File content: {}", content);

    drop(file);

    unsafe {
        esp_vfs_fat_sdcard_unmount(mount_point.as_ptr(), card_handle);
    }

    let _ = test_display();
}
