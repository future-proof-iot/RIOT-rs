pub use {const_panic, konst};

macro_rules! define_env_with_default_macro {
    ($macro_name:ident, $parse_fn_name:ident, $output_type_name:literal) => {
        #[macro_export]
        macro_rules! $macro_name {
            // $doc is currently unused
            ($env_var:literal, $default:expr, $doc:literal) => {
                if let Some(str_value) = option_env!($env_var) {
                    if let Ok(value) = $crate::env::konst::primitive::$parse_fn_name(str_value) {
                        value
                    } else {
                        $crate::env::const_panic::concat_panic!(
                            "Could not parse environment variable `",
                            $env_var,
                            "=",
                            str_value,
                            "` as ",
                            $output_type_name,
                        );
                    }
                } else {
                    $default
                }
            };
        }
    };
}

define_env_with_default_macro!(usize_from_env_or, parse_usize, "a usize");
define_env_with_default_macro!(u8_from_env_or, parse_u8, "a u8");

#[macro_export]
macro_rules! str_from_env_or {
    // $doc is currently unused
    ($env_var:literal, $default:expr, $doc:literal) => {
        if let Some(str_value) = option_env!($env_var) {
            str_value
        } else {
            $default
        }
    };
}
