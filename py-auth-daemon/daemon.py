import socket
import os
import msgpack
from ig_client import IGClient
from instagrapi.exceptions import ChallengeRequired, CheckpointRequired

# Fixed path for the Unix domain socket
SOCKET_PATH = "/tmp/ig-reels.sock"

def start_daemon():
    # Remove socket file if it exists
    if os.path.exists(SOCKET_PATH):
        os.remove(SOCKET_PATH)

    # Create a Unix domain socket server
    server = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    server.bind(SOCKET_PATH)
    server.listen(1)
    print(f"Daemon listening on {SOCKET_PATH}")

    # Initialize the IGClient (we'll load session or login when needed)
    # For simplicity, we assume credentials are set via environment variables
    # In a real scenario, we might want to prompt or use a config file, but for now
    # we'll use environment variables INSTAGRAM_USERNAME and INSTAGRAM_PASSWORD
    ig_client = IGClient()
    username = os.getenv("INSTAGRAM_USERNAME")
    password = os.getenv("INSTAGRAM_PASSWORD")

    if not username or not password:
        print("Error: INSTAGRAM_USERNAME and INSTAGRAM_PASSWORD environment variables must be set")
        server.close()
        return

    try:
        # Try to load session or login
        ig_client.load_or_login(username, password)
        print("IG client initialized and logged in")
    except Exception as e:
        print(f"Failed to initialize IG client: {e}")
        server.close()
        return

    try:
        while True:
            conn, addr = server.accept()
            with conn:
                print(f"Connected by {addr}")
                data = conn.recv(4096)
                if not data:
                    continue
                try:
                    request = msgpack.unpackb(data, raw=False)
                    print(f"Received request: {request}")
                    cmd = request.get("cmd")
                    if cmd == "ping":
                        response = {"status": "ok"}
                    elif cmd == "get_reels":
                        count = request.get("count", 1)
                        try:
                            reels = ig_client.get_reels_batch(count)
                            response = {"reels": reels}
                        except Exception as e:
                            # Handle specific exceptions like challenge_required
                            if isinstance(e, (ChallengeRequired, CheckpointRequired)):
                                response = {"error": "challenge_required", "detail": str(e)}
                            else:
                                response = {"error": "internal_error", "detail": str(e)}
                    else:
                        response = {"error": "unknown_command", "detail": f"Unknown command: {cmd}"}
                except Exception as e:
                    response = {"error": "invalid_request", "detail": str(e)}
                
                # Send response
                conn.sendall(msgpack.packb(response, use_bin_type=True))
                print(f"Sent response: {response}")
    except KeyboardInterrupt:
        print("\nShutting down daemon...")
    finally:
        server.close()
        if os.path.exists(SOCKET_PATH):
            os.remove(SOCKET_PATH)

if __name__ == "__main__":
    start_daemon()
