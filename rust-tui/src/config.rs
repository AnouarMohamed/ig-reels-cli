use std::env;
use std::path::PathBuf;

/// Configuration for the IG-Reels-CLI application.
///
/// Values are read from environment variables with sensible defaults where applicable.
#[derive(Debug, Clone)]
pub struct Config {
    /// Path to the Unix domain socket for communication with the Python gateway.
    pub socket_path: PathBuf,
    /// Path to the session file used by the Python gateway.
    pub session_path: PathBuf,
    /// Directory for caching media files.
    pub cache_dir: PathBuf,
    /// Directory for log files.
    pub log_dir: PathBuf,
    /// Render frames per second.
    pub render_fps: u16,
    /// Maximum terminal columns for rendering.
    pub max_cols: u16,
    /// Maximum terminal rows for rendering.
    pub max_rows: u16,
    /// Fallback aspect ratio for cells (height/width) when terminal pixel dimensions are unavailable.
    pub cell_aspect_fallback: f32,
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// # Panics
    /// Panics if any required environment variable is missing or has an invalid value.
    pub fn from_env() -> Self {
        // Shared paths (required, supplied by launcher)
        let socket_path = Self::req_path("IG_REELS_SOCKET_PATH");
        let session_path = Self::req_path("IG_REELS_SESSION_PATH");
        let cache_dir = Self::req_path("IG_REELS_CACHE_DIR");
        let log_dir = Self::req_path("IG_REELS_LOG_DIR");

        // Optional renderer configuration with defaults
        let render_fps = Self::parse_u16(
            "IG_REELS_RENDER_FPS",
            15,
            |v| v > 0,
            "must be positive",
        );
        let max_cols = Self::parse_u16(
            "IG_REELS_MAX_COLS",
            120,
            |v| v > 0,
            "must be positive",
        );
        let max_rows = Self::parse_u16(
            "IG_REELS_MAX_ROWS",
            60,
            |v| v > 0,
            "must be positive",
        );
        let cell_aspect_fallback = Self::parse_f32(
            "IG_REELS_CELL_ASPECT_FALLBACK",
            2.0,
            |v| v > 0.0,
            "must be positive",
        );

        Self {
            socket_path,
            session_path,
            cache_dir,
            log_dir,
            render_fps,
            max_cols,
            max_rows,
            cell_aspect_fallback,
        }
    }

    /// Required path environment variable.
    fn req_path(var: &str) -> PathBuf {
        env::var(var)
            .unwrap_or_else(|_| panic!("{} is not set", var))
            .into()
    }

    /// Parse a u16 environment variable with a default and validation.
    fn parse_u16<F>(key: &str, default: u16, validate: F, err_msg: &str) -> u16
    where
        F: Fn(u16) -> bool,
    {
        match env::var(key) {
            Ok(val) => val
                .parse()
                .unwrap_or_else(|_| panic!("{} must be an unsigned integer", key)),
            Err(_) => default,
        }
        .tap(|v| {
            if !validate(*v) {
                panic!("{} {}: {}", key, v, err_msg);
            }
        })
    }

    /// Parse a f32 environment variable with a default and validation.
    fn parse_f32<F>(key: &str, default: f32, validate: F, err_msg: &str) -> f32
    where
        F: Fn(f32) -> bool,
    {
        match env::var(key) {
            Ok(val) => val
                .parse()
                .unwrap_or_else(|_| panic!("{} must be a floating-point number", key)),
            Err(_) => default,
        }
        .tap(|v| {
            if !validate(*v) {
                panic!("{} {}: {}", key, v, err_msg);
            }
        })
    }
}

/// Extension trait for tapping (inspecting) a value without consuming it.
trait Tap<T> {
    fn tap<F>(self, f: F) -> Self
    where
        F: FnOnce(&T);
}

impl<T> Tap<T> for T {
    fn tap<F>(self, f: F) -> Self
    where
        F: FnOnce(&T),
    {
        f(&self);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn set_env(var: &str, val: &str) {
        env::set_var(var, val);
    }

    fn unset_env(var: &str) {
        env::remove_var(var);
    }

    fn set_renderer_defaults() {
        set_env("IG_REELS_RENDER_FPS", "15");
        set_env("IG_REELS_MAX_COLS", "120");
        set_env("IG_REELS_MAX_ROWS", "60");
        set_env("IG_REELS_CELL_ASPECT_FALLBACK", "2.0");
    }

    fn set_required_paths() {
        set_env("IG_REELS_SOCKET_PATH", "/tmp/test.sock");
        set_env("IG_REELS_SESSION_PATH", "/tmp/session.json");
        set_env("IG_REELS_CACHE_DIR", "/tmp/cache");
        set_env("IG_REELS_LOG_DIR", "/tmp/logs");
    }

    #[test]
    fn test_config_from_env_with_defaults() {
        set_renderer_defaults();
        set_required_paths();

        let config = Config::from_env();
        assert_eq!(config.socket_path, PathBuf::from("/tmp/test.sock"));
        assert_eq!(config.session_path, PathBuf::from("/tmp/session.json"));
        assert_eq!(config.cache_dir, PathBuf::from("/tmp/cache"));
        assert_eq!(config.log_dir, PathBuf::from("/tmp/logs"));
        assert_eq!(config.render_fps, 15);
        assert_eq!(config.max_cols, 120);
        assert_eq!(config.max_rows, 60);
        assert_eq!(config.cell_aspect_fallback, 2.0);
    }

    #[test]
    fn test_config_from_env_with_custom_values() {
        set_renderer_defaults();
        set_required_paths();
        set_env("IG_REELS_RENDER_FPS", "30");
        set_env("IG_REELS_MAX_COLS", "80");
        set_env("IG_REELS_MAX_ROWS", "40");
        set_env("IG_REELS_CELL_ASPECT_FALLBACK", "1.5");

        let config = Config::from_env();
        assert_eq!(config.socket_path, PathBuf::from("/tmp/test.sock"));
        assert_eq!(config.session_path, PathBuf::from("/tmp/session.json"));
        assert_eq!(config.cache_dir, PathBuf::from("/tmp/cache"));
        assert_eq!(config.log_dir, PathBuf::from("/tmp/logs"));
        assert_eq!(config.render_fps, 30);
        assert_eq!(config.max_cols, 80);
        assert_eq!(config.max_rows, 40);
        assert_eq!(config.cell_aspect_fallback, 1.5);
    }

    #[test]
    #[should_panic(expected = "IG_REELS_RENDER_FPS")]
    fn test_config_invalid_fps_negative() {
        set_renderer_defaults();
        set_required_paths();
        set_env("IG_REELS_RENDER_FPS", "0");
        let _ = Config::from_env();
    }

    #[test]
    #[should_panic(expected = "IG_REELS_MAX_COLS")]
    fn test_config_invalid_max_cols_zero() {
        set_renderer_defaults();
        set_required_paths();
        set_env("IG_REELS_MAX_COLS", "0");
        let _ = Config::from_env();
    }

    #[test]
    #[should_panic(expected = "IG_REELS_MAX_ROWS")]
    fn test_config_invalid_max_rows_zero() {
        set_renderer_defaults();
        set_required_paths();
        set_env("IG_REELS_MAX_ROWS", "0");
        let _ = Config::from_env();
    }

    #[test]
    #[should_panic(expected = "IG_REELS_CELL_ASPECT_FALLBACK")]
    fn test_config_invalid_aspect_zero() {
        set_renderer_defaults();
        set_required_paths();
        set_env("IG_REELS_CELL_ASPECT_FALLBACK", "0.0");
        let _ = Config::from_env();
    }

    #[test]
    #[should_panic(expected = "IG_REELS_SOCKET_PATH")]
    fn test_config_missing_socket_path() {
        set_required_paths();
        unset_env("IG_REELS_SOCKET_PATH");
        set_renderer_defaults();
        let _ = Config::from_env();
    }
}
