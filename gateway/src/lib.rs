pub use sqlite_database::SqliteDatabaseGateway;

pub mod github;
mod sqlite_database;

macro_rules! error_and_log {
    ($msg:literal) => {
        {
            ::tracing::error!($msg);
            ::anyhow::anyhow!($msg)
        }
    };
    ($fmt:expr, $($arg:tt)*) => {
        {
            ::tracing::error!($fmt, $($arg)*);
            ::anyhow::anyhow!($fmt, $($arg)*)
        }
    };
}
pub(crate) use error_and_log;
