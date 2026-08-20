import socket
import msgpack
import sys

SOCKET_PATH = "/tmp/ig-reels.sock"

def send_request(request):
    client = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    try:
        client.connect(SOCKET_PATH)
        client.sendall(msgpack.packb(request, use_bin_type=True))
        response = client.recv(4096)
        return msgpack.unpackb(response, raw=False)
    except Exception as e:
        print(f"Error sending request: {e}")
        return None
    finally:
        client.close()

if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "ping":
        response = send_request({"cmd": "ping"})
        print("Ping response:", response)
    elif len(sys.argv) > 1 and sys.argv[1] == "get_reels":
        count = int(sys.argv[2]) if len(sys.argv) > 2 else 1
        response = send_request({"cmd": "get_reels", "count": count})
        if response:
            if "reels" in response:
                print(f"Got {len(response['reels'])} reels:")
                for i, reel in enumerate(response['reels']):
                    print(f"  {i+1}. ID: {reel['id']}, Username: {reel['username']}, Likes: {reel['like_count']}")
            else:
                print("Error in response:", response)
        else:
            print("No response")
    else:
        print("Usage:")
        print("  python test_client.py ping")
        print("  python test_client.py get_reels [count]")
