extern crate env_logger;
extern crate log_panics;
extern crate log;
extern crate android_logger;

use self::env_logger::LogBuilder;
use self::log::{Record, LevelFilter, Level};
use std::env;
use std::sync::{Once, ONCE_INIT};
use self::android_logger::Filter;

pub struct LoggerUtils {}

static LOGGER_INIT: Once = ONCE_INIT;

impl LoggerUtils {
    pub fn init() {
        //Starts logging the panic messages using the system logger.
        LOGGER_INIT.call_once(|| {

            log_panics::init(); //Logging of panics is essential for android. As android does not log to stdout for native code
            if cfg!(target_os = "android") {
                //Set logging to off when deploying production android app.
                android_logger::init_once(
                    Filter::default().with_min_level(log::Level::Trace)
                );
                info!("Logging for Android");
            } else {
//                let format = |record: &Record| {
//                    format!("{:>5}|{:<30}|{:>35}:{:<4}| {}", record.level(), record.target(), record.file().get_or_insert(""), record.line().get_or_insert(0), record.args())
//                };
                let mut builder = LogBuilder::new();
//                builder.format(format);

                if env::var("RUST_LOG").is_ok() {
                    builder.parse(&env::var("RUST_LOG").unwrap());
                }

                builder.init().unwrap();
            }
        });
    }
}

#[macro_export]
macro_rules! try_log {
    ($expr:expr) => (match $expr {
        Ok(val) => val,
        Err(err) => {
            error!("try_log! | {}", err);
            return Err(From::from(err))
        }
    })
}

macro_rules! _map_err {
    ($lvl:expr, $expr:expr) => (
        |err| {
            log!($lvl, "{} - {}", $expr, err);
            err
        }
    );
    ($lvl:expr) => (
        |err| {
            log!($lvl, "{}", err);
            err
        }
    )
}

#[macro_export]
macro_rules! map_err_err {
    () => ( _map_err!(::log::Level::Error) );
    ($($arg:tt)*) => ( _map_err!(::log::Level::Error, $($arg)*) )
}

#[macro_export]
macro_rules! map_err_trace {
    () => ( _map_err!(::log::Level::Trace) );
    ($($arg:tt)*) => ( _map_err!(::log::Level::Trace, $($arg)*) )
}

#[macro_export]
macro_rules! map_err_info {
    () => ( _map_err!(::log::Level::Info) );
    ($($arg:tt)*) => ( _map_err!(::log::Level::Info, $($arg)*) )
}
