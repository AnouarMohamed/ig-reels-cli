mod config;

use config::Config;

fn main() {
    let config = Config::from_env();
    println!("IG-Reels-CLI v1.0");
    println!(
        "Config: socket={}, session={}, cache={}, log={}, FPS={}, cols={}, rows={}, aspect={}",
        config.socket_path.display(),
        config.session_path.display(),
        config.cache_dir.display(),
        config.log_dir.display(),
        config.render_fps,
        config.max_cols,
        config.max_rows,
        config.cell_aspect_fallback
    );
}
