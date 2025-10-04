// Library exports for testing
pub mod analysis;
pub mod app;
pub mod config;
pub mod db;
pub mod history;
pub mod ui;

// Re-export commonly used types for tests
pub use app::{App, AppStats, Tab};
pub use config::Config;
pub use db::Database;
pub use history::{Command, PackageRef};
