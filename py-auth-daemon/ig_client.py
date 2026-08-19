import json
import os
from instagrapi import Client
from instagrapi.exceptions import ChallengeRequired, CheckpointRequired, LoginRequired
import msgpack

class IGClient:
    def __init__(self, session_file="session.json"):
        self.session_file = session_file
        self.client = Client()

    def login(self, username, password):
        """Login with username and password, and save session."""
        try:
            self.client.login(username, password)
            self.client.dump_settings(self.session_file)
            return True
        except (ChallengeRequired, CheckpointRequired) as e:
            # Return the exception for the daemon to handle
            raise e
        except Exception as e:
            raise e

    def load_or_login(self, username=None, password=None):
        """Load session from file if exists, otherwise login."""
        if os.path.exists(self.session_file):
            try:
                self.client.load_settings(self.session_file)
                # Verify the session is still valid
                self.client.get_timeline_feed()
                return True
            except (LoginRequired, ChallengeRequired, CheckpointRequired):
                # Session is invalid, we need to login again
                pass
            except Exception:
                pass

        if username is None or password is None:
            raise ValueError("Username and password required for login when no valid session exists")
        return self.login(username, password)

    def get_reels_batch(self, count):
        """Get a batch of reels (explore feed). Returns list of dicts."""
        try:
            # Get explore reels (this is the equivalent of the explore page reels)
            reels = self.client.get_explore_tab(reels_count=count)
            results = []
            for reel in reels:
                # We need to extract the required fields
                # Note: instagrapi's Reel media type has different attributes
                # We'll try to get the video URL, caption, username, like count, and id
                video_url = None
                if reel.video_url:
                    video_url = reel.video_url
                else:
                    # If no video_url, try to get from resources (if it's a carousel with video)
                    # For simplicity, we skip non-video reels
                    continue

                results.append({
                    "id": reel.id,
                    "video_url": video_url,
                    "caption": reel.caption_text if reel.caption_text else "",
                    "username": reel.user.username,
                    "like_count": reel.like_count
                })
            return results
        except Exception as e:
            # Re-raise the exception for the daemon to handle
            raise e

# For testing purposes, if this file is run directly
if __name__ == "__main__":
    # This is just a simple test - in practice, the daemon will use this class
    import sys
    if len(sys.argv) != 3:
        print("Usage: python ig_client.py <username> <password>")
        sys.exit(1)
    username, password = sys.argv[1], sys.argv[2]
    client = IGClient()
    try:
        if client.load_or_login(username, password):
            print("Logged in successfully")
            reels = client.get_reels_batch(5)
            print(f"Got {len(reels)} reels")
            for reel in reels:
                print(f"ID: {reel['id']}, Username: {reel['username']}, Likes: {reel['like_count']}")
    except Exception as e:
        print(f"Error: {e}")
