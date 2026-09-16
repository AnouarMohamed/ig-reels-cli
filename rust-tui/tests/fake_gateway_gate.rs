use std::path::PathBuf;
use std::process::{Child, Command};
use std::time::Duration;
use tokio::time::sleep;

use rust_tui::ipc::{send_request, Request};

struct GatewayProcess {
    child: Child,
    sock_path: PathBuf,
}

impl GatewayProcess {
    fn start(sock_path: PathBuf) -> Self {
        if sock_path.exists() {
            let _ = std::fs::remove_file(&sock_path);
        }

        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf();
        let script_path = repo_root.join("scripts").join("fake-gateway.py");

        let child = Command::new("python3")
            .arg(&script_path)
            .arg("--socket-path")
            .arg(&sock_path)
            .spawn()
            .expect("Failed to spawn fake-gateway.py");

        Self { child, sock_path }
    }
}

impl Drop for GatewayProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        if self.sock_path.exists() {
            let _ = std::fs::remove_file(&self.sock_path);
        }
    }
}

#[tokio::test]
async fn test_gate_ping_and_get_reels_fake_gateway() {
    let sock_path = PathBuf::from(format!(
        "/tmp/test_gate_{}_{}.sock",
        std::process::id(),
        rand_id()
    ));

    let _gateway = GatewayProcess::start(sock_path.clone());

    // Wait up to 3 seconds for socket to exist
    let mut ready = false;
    for _ in 0..60 {
        if sock_path.exists() {
            ready = true;
            break;
        }
        sleep(Duration::from_millis(50)).await;
    }
    assert!(ready, "fake-gateway.py socket did not appear in time");

    // Ping gate
    let ping_req = Request::ping("gate-ping-100");
    let ping_resp = send_request(&sock_path, &ping_req)
        .await
        .expect("ping fake gateway failed");
    assert!(ping_resp.ok);
    assert_eq!(ping_resp.request_id, "gate-ping-100");

    let ping_res = ping_resp.result.expect("ping result");
    assert_eq!(ping_res["service"], "ig-gateway");
    assert_eq!(ping_res["status"], "ok");

    // Get reels gate
    let reels_req = Request::get_reels("gate-reels-200", 2);
    let reels_resp = send_request(&sock_path, &reels_req)
        .await
        .expect("get_reels fake gateway failed");
    assert!(reels_resp.ok);
    assert_eq!(reels_resp.request_id, "gate-reels-200");

    let reels_res = reels_resp.result.expect("reels result");
    let items = reels_res["items"].as_array().expect("items array");
    assert_eq!(items.len(), 2);
    assert!(items[0]["video_url"]
        .as_str()
        .unwrap()
        .starts_with("https://example.com/"));
}

fn rand_id() -> u32 {
    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNTER: AtomicU32 = AtomicU32::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}
