use lazy_static::lazy_static;
use std::error::Error;
use std::ffi::CStr;

#[allow(dead_code)]
mod raw_bindings;

fn safe_c_str_to_string(c_str: &'static [u8]) -> Result<&'static str, Box<dyn Error>> {
    Ok({ CStr::from_bytes_with_nul(c_str) }?.to_str()?)
}

lazy_static! {
    pub static ref BLE_MAIN_SERVICE_LOCAL_NAME: &'static str =
        safe_c_str_to_string(raw_bindings::BLE_MAIN_SERVICE_LOCAL_NAME)
            .expect("Invalid UTF-8 for BLE_MAIN_SERVICE_LOCAL_NAME");
    pub static ref BLE_MAIN_SENSOR_SERVICE: &'static str =
        safe_c_str_to_string(raw_bindings::BLE_MAIN_SENSOR_SERVICE)
            .expect("Invalid UTF-8 for BLE_MAIN_SENSOR_SERVICE");
    pub static ref BLE_MAIN_SENSOR_STREAM_CHAR: &'static str =
        safe_c_str_to_string(raw_bindings::BLE_MAIN_SENSOR_STREAM_CHAR)
            .expect("Invalid UTF-8 for BLE_MAIN_SENSOR_STREAM_CHAR");
    pub static ref BLE_MAIN_SENSOR_CO2_CALIBRATION_CHAR: &'static str =
        safe_c_str_to_string(raw_bindings::BLE_MAIN_SENSOR_CO2_CALIBRATION_CHAR)
            .expect("Invalid UTF-8 for BLE_MAIN_SENSOR_STREAM_CHAR");
    pub static ref BLE_MAIN_SENSOR_TEMP_CALIBRATION_CHAR: &'static str =
        safe_c_str_to_string(raw_bindings::BLE_MAIN_SENSOR_TEMP_CALIBRATION_CHAR)
            .expect("Invalid UTF-8 for BLE_MAIN_SENSOR_ACTION_CHAR");
    pub static ref BLE_MAIN_SENSOR_CALIBRATE_CO2: &'static str =
        safe_c_str_to_string(raw_bindings::BLE_MAIN_SENSOR_CALIBRATE_CO2)
            .expect("Invalid UTF-8 for BLE_MAIN_SENSOR_CALIBRATE_CO2");
    pub static ref BLE_MAIN_SENSOR_CALIBRATE_TEMPERATURE: &'static str =
        safe_c_str_to_string(raw_bindings::BLE_MAIN_SENSOR_CALIBRATE_TEMPERATURE)
            .expect("Invalid UTF-8 for BLE_MAIN_SENSOR_CALIBRATE_TEMPERATURE");
    pub static ref BLE_MAIN_SENSOR_CALIBRATE_HUMIDITY: &'static str =
        safe_c_str_to_string(raw_bindings::BLE_MAIN_SENSOR_CALIBRATE_HUMIDITY)
            .expect("Invalid UTF-8 for BLE_MAIN_SENSOR_CALIBRATE_HUMIDITY");
    pub static ref BLE_WINDOW_SERVICE_LOCAL_NAME: &'static str =
        safe_c_str_to_string(raw_bindings::BLE_WINDOW_SERVICE_LOCAL_NAME)
            .expect("Invalid UTF-8 for BLE_WINDOW_SERVICE_LOCAL_NAME");
    pub static ref BLE_WINDOW_SENSOR_SERVICE: &'static str =
        safe_c_str_to_string(raw_bindings::BLE_WINDOW_SENSOR_SERVICE)
            .expect("Invalid UTF-8 for BLE_WINDOW_SENSOR_SERVICE");
    pub static ref BLE_WINDOW_SENSOR_READ_CHAR: &'static str =
        safe_c_str_to_string(raw_bindings::BLE_WINDOW_SENSOR_READ_CHAR)
            .expect("Invalid UTF-8 for BLE_WINDOW_SENSOR_STREAM_CHAR");
}
