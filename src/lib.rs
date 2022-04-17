#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod test {
    use crate::{cef_app_t, cef_initialize, cef_life_span_handler_t, cef_log_severity_t_LOGSEVERITY_WARNING, cef_main_args_t, cef_settings_t, GetModuleHandleA};

    #[test]
    fn it_works() {
        unsafe {
            let args = cef_main_args_t {
                instance: GetModuleHandleA(std::ptr::null()),
            };
            let handler = cef_life_span_handler_t::default();

            let mut app = cef_app_t::default();

            let settings = cef_settings_t {
                size: std::mem::size_of::<cef_settings_t>(),
                log_severity: cef_log_severity_t_LOGSEVERITY_WARNING,
                ..Default::default()
            };

            cef_initialize(&args, &settings, &mut app, std::ptr::null_mut());
        }
    }
}