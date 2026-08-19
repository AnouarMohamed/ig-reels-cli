Data Flow: User presses 'Next' (n, →, or j) to advance to the next reel.

Assumptions:
- The app is already playing reel N from a prefetched queue.
- Reel N+1 has been prefetched (downloaded, decoded) or will be triggered by this action.
- State is managed in app.rs; modules communicate via async channels or shared state.

Step-by-step:

1. Input Capture (input.rs)
   - crossterm::Event::Key received for 'Next' keybinding.
   - input.rs sends an AppEvent::Next via a tokio::sync::broadcast or channel to app.rs.

2. Event Handling (app.rs)
   - app.rs receives AppEvent::Next.
   - Checks internal reel queue (VecDeque<ReelMetadata>) for an available next reel.
   - If queue empty, signals fetch.rs to replenish (but prefetch should keep at least one ahead).
   - Sets next_reel_id and triggers transition state.

3. Transition Initiation (app.rs)
   - Signals current playback tasks (decode, render, audio) to stop gracefully via shutdown Sender.
   - Waits for current frame/audio streams to end (or drops them).
   - Clears any buffered frames/audio for reel N.

4. Ensure Next Reel Ready (fetch.rs → download.rs)
   - fetch.rs receives a request (via app state) to ensure reel N+1 is available.
   - Checks download.rs cache (HashSet of completed downloads or tmp-cache/<id>.mp4 existence).
   - If MP4 missing, calls download.rs::fetch_video(url) to start async download.

5. Video Download (download.rs)
   - Given video_url and reel ID, constructs path tmp-cache/<id>.mp4.
   - Uses reqwest (or surf) to stream GET request, writing to temp file then renaming.
   - On completion, notifies fetch.rs (via callback or channel) that reel N+1 is ready.
   - Updates shared state so app.rs knows the file exists.

6. Frame Extraction (decode.rs)
   - app.rs spawns a decode task for reel N+1.
   - decode.rs opens tmp-cache/<id>.mp4 with ffmpeg-next (VideoDecoder).
   - Configures to output raw RGB24 frames at target fps (e.g., 30).
   - Extracts frames iteratively, sending each RGB buffer via an async Sender to render.rs.

7. Audio Preparation (audio.rs)
   - In parallel, audio.rs opens same MP4, extracts audio track via ffmpeg-next (AudioDecoder).
   - Decodes to PCM f32, resamples to rodio's expected sample rate (48kHz) if needed.
   - Sends audio samples to rodio::OutputStream via an iterator/Sink.
   - Synchronization: audio start is delayed to align with first video frame timestamp (both start at 0).

8. Rendering Loop (render.rs)
   - render.rs receives RGB frame buffer (Vec<u8>, width×height×3).
   - Converts to terminal output:
       * If Kitty protocol available (checked at start via env), encodes each row as a graphics packet.
       * Else uses ANSI truecolor half-block: for each pixel row pair (top,bottom),
         output '▀' with fg = top RGB, bg = bottom RGB.
   - Writes resulting bytes to stdout via crossterm::queue!(crossterm::Print, ...) then flush.
   - Each frame is cleared before next (using crossterm::Clear or by overwriting).

9. Playback Coordination (app.rs)
   - app.rs drives a fixed‑time loop synced to video fps.
   - On each tick:
        - If a new video frame is available, passes it to render.rs.
        - Audio plays continuously via rodio (no per‑frame action needed).
        - Sleeps until next frame deadline.
   - Handles pause/resume by toggling a flag that skips frame advances but keeps audio paused via rodio::Sink::set_pause.

10. End of Reel
    - When decode.rs signals EOF (no more frames):
        - App.rs waits for audio to finish (or cuts off at same duration).
        - Increments reel index, loops back to step 2 for next reel in queue.
        - If queue now low (<2 reels ahead), fetch.rs triggers background fetch of next batch.

Note: Prefetch logic runs continuously in background: download.rs::prefetch_ahead() watches the queue and initiates download of reel N+2 when N+1 is being displayed, ensuring at least two reels are ready ahead of playback.