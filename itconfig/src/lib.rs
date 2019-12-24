//! # itconfig
//!
//! Simple configuration with macro for rust application.
//!
//!
//! ## Example usage
//!
//! ```rust
//! #[macro_use] extern crate itconfig;
//! // use dotenv::dotenv;
//!
//! config! {
//!     DEBUG: bool => true,
//!     HOST: String => "127.0.0.1".to_string(),
//! }
//!
//! fn main () {
//!     // dotenv().ok();
//!     cfg::init();
//!     assert_eq!(cfg::HOST(), String::from("127.0.0.1"));
//! }
//! ```

#[doc(hidden)]
macro_rules! __impl_from_for_numbers {
    (
        $($ty:ty),+
    ) => {
        $(
            impl From<EnvValue> for $ty {
                fn from(env: EnvValue) -> Self {
                    env.0.parse::<Self>().unwrap()
                }
            }
        )*
    }
}


#[derive(Debug)]
#[doc(hidden)]
struct EnvValue(String);

impl EnvValue {
    pub fn new(string: String) -> Self {
        Self(string)
    }
}

__impl_from_for_numbers![
    i8, i16, i32, i64, i128, isize,
    u8, u16, u32, u64, u128, usize,
    f32, f64
];

impl From<EnvValue> for bool {
    fn from(env: EnvValue) -> Self {
        match env.0.to_lowercase().as_str() {
            "true" | "1" | "t" | "on" => true,
            _ => false,
        }
    }
}

impl From<String> for EnvValue {
    fn from(val: String) -> Self {
        Self(val)
    }
}

impl From<EnvValue> for String {
    fn from(env: EnvValue) -> Self {
        env.0
    }
}


/// Creates new public mod with function fo get each environment variable of mapping.
///
/// All variables are required and program will panic if some variables haven't value, but you
/// can add default value for specific variable.
///
/// Example usage
/// -------------
///
/// ```rust
/// # #[macro_use] extern crate itconfig;
/// config! {
///     DATABASE_URL: String,
/// }
/// # fn main () {}
/// ```
///
/// Config with default value
///
/// ```rust
/// # #[macro_use] extern crate itconfig;
/// config! {
///     DATABASE_URL: String,
///     HOST: String => "127.0.0.1".to_string(),
/// }
/// # fn main () {}
/// ```
///
/// By default itconfig lib creates module with 'cfg' name. But you can use simple meta instruction
/// if you want to rename module. In the example below we renamed module to 'configuration'
///
/// ```rust
/// # #[macro_use] extern crate itconfig;
/// config! {
///     #![mod_name = configuration]
///
///     DEBUG: bool,
/// }
/// # fn main () {}
/// ```
///
///
/// This module will also contain helper method:
///
/// `init`
/// ------
///
/// If you miss some required variables your application will panic at startup.
/// Run this at the main function for check all required variables without default value.
///
/// ```rust
/// #[macro_use] extern crate itconfig;
/// // use dotenv::dotenv;
///
/// config! {
///     DEBUG: bool => true,
///     HOST: String => "127.0.0.1".to_string(),
/// }
///
/// fn main () {
///     // dotenv().ok();
///     cfg::init();
///     assert_eq!(cfg::HOST(), String::from("127.0.0.1"));
/// }
/// ```
///
#[macro_export]
macro_rules! config {
    ($($tokens:tt)*) => {
        __itconfig_parse_module! {
            tokens = [$($tokens)*],
            name = cfg,
        }
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! __itconfig_invalid_syntax {
    () => {
        compile_error!(
            "Invalid `config!` syntax. Please see the `config!` macro docs for more info."
        );
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __itconfig_parse_module {
    // Find module name
    (
        tokens = [
            #![mod_name = $mod_name:ident]
            $($rest:tt)*
        ],
        name = $ignore:tt,
    ) => {
        __itconfig_parse_module! {
            tokens = [$($rest)*],
            name = $mod_name,
        }
    };

    // Done parsing module
    (
        tokens = $tokens:tt,
        name = $name:tt,
    ) => {
        __itconfig_parse_variables! {
            tokens = $tokens,
            variables = [],
            module = {
                name = $name,
            },
        }
    };

    // Invalid syntax
    ($($tokens:tt)*) => {
        __itconfig_invalid_syntax!();
    };
}


#[macro_export]
#[doc(hidden)]
macro_rules! __itconfig_parse_variables {
    // Find variable with default value
    (
        tokens = [
            $name:ident : $ty:ty => $default:expr,
            $($rest:tt)*
        ],
        $($args:tt)*
    ) => {
        __itconfig_parse_variables! {
            current_variable = {
                name = $name,
                ty = $ty,
                env_name = stringify!($name),
                default = $default,
            },
            tokens = [$($rest)*],
            $($args)*
        }
    };

    // Find variable without default value
    (
        tokens = [
            $name:ident : $ty:ty,
            $($rest:tt)*
        ],
        $($args:tt)*
    ) => {
        __itconfig_parse_variables! {
            current_variable = {
                name = $name,
                ty = $ty,
                env_name = stringify!($name),
            },
            tokens = [$($rest)*],
            $($args)*
        }
    };

    // Done parsing variable
    (
        current_variable = {
            $($current_variable:tt)*
        },
        tokens = $tokens:tt,
        variables = [$($variables:tt,)*],
        $($args:tt)*
    ) => {
        __itconfig_parse_variables! {
            tokens = $tokens,
            variables = [$($variables,)* { $($current_variable)* },],
            $($args)*
        }
    };

    // Done parsing all variables
    (
        tokens = [],
        $($args:tt)*
    ) => {
        __itconfig_impl!($($args)*);
    };

    // Invalid syntax
    ($($tokens:tt)*) => {
        __itconfig_invalid_syntax!();
    };
}


#[macro_export]
#[doc(hidden)]
macro_rules! __itconfig_impl {
    (
        variables = [$({
            name = $name:ident,
            $($variable:tt)*
        },)+],
        module = {
            name = $mod_name:ident,
        },
    ) => {
        pub mod $mod_name {
            #![allow(non_snake_case)]
            use std::env;
            use $crate::EnvValue;

            pub fn init() {
                $($name();)+
            }

            $(__itconfig_variable! {
                name = $name,
                $($variable)*
            })+
        }
    };
}


#[macro_export]
#[doc(hidden)]
macro_rules! __itconfig_variable {
    // Add method with default value
    (
        name = $name:ident,
        ty = $ty:ty,
        env_name = $env_name:expr,
        default = $default:expr,
    ) => {
        pub fn $name() -> $ty {
            env::var($env_name)
                .map(|val| EnvValue::from(val).into())
                .unwrap_or_else(|_| $default)
        }
    };

    // Add method without default value
    (
        name = $name:ident,
        ty = $ty:ty,
        env_name = $env_name:expr,
    ) => {
        pub fn $name() -> $ty {
            env::var($env_name)
                .map(|val| EnvValue::from(val).into())
                .unwrap_or_else(|_| {
                    panic!(format!(r#"Cannot read "{}" environment variable"#, $env_name))
                })

        }
    };
}
