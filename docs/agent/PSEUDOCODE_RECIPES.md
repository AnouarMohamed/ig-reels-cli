# Pseudocode Recipes — Algorithm Guidance, Not Library API

These are logic recipes. They intentionally avoid pretending exact dependency method names. Verify real APIs before coding.

## 1. Read one framed IPC message

```text
read_exact(4 bytes) -> header
len = big_endian_u32(header)
if len == 0 or len > MAX_IPC_PAYLOAD: error
allocate payload[len]
read_exact(payload)
decode MessagePack map
validate protocol_version/request shape
return message
```

Do not call one generic `read()` and assume it returns one message.

## 2. Write one framed IPC message

```text
payload = msgpack_named_map_encode(message)
if payload.len == 0 or > limit: error
header = u32(payload.len).to_be_bytes()
write_all(header)
write_all(payload)
flush if boundary requires
```

## 3. Geometry

```text
function choose_cell_ratio(metrics):
    if pixel metrics > 0:
        cw = pixel_width / cols as float
        ch = pixel_height / rows as float
        ratio = ch / cw
        if ratio finite and 1.0 <= ratio <= 4.0:
            return ratio
    return FALLBACK  # 2.0

function geometry(source_dar, cols, rows, cell_ratio):
    available_rows = rows - STATUS_ROWS
    max_cell_cols = min(cols, MAX_COLS)
    max_cell_rows = min(available_rows, MAX_ROWS)
    max_sample_w = max_cell_cols
    max_sample_h = 2 * max_cell_rows
    max_h_even = max_sample_h if even else max_sample_h - 1

    ratio = source_dar * cell_ratio / 2
    raw_h = max_sample_w / ratio

    if raw_h <= max_h_even:
        W = max_sample_w
        H = nearest_even(raw_h, max_h_even)
    else:
        H = max_h_even
        W = clamp(round(H * ratio), 1, max_sample_w)

    cell_cols = W
    cell_rows = H / 2
    origin_x = (cols - cell_cols) / 2
    origin_y = (available_rows - cell_rows) / 2
    return geometry
```

## 4. Render one RGB frame

```text
validate height even
validate bytes.len == width*height*3
out = byte_buffer_with_capacity(...)
append reset

for cell_y in 0 .. height/2:
    append cursor_move(origin_x, origin_y + cell_y)
    fg_state = unknown or carried according to chosen encoder policy
    bg_state = unknown or carried according to chosen encoder policy

    for x in 0 .. width:
        top = rgb_at(x, cell_y*2)
        bottom = rgb_at(x, cell_y*2+1)
        if top != current_fg: append FG truecolor; current_fg = top
        if bottom != current_bg: append BG truecolor; current_bg = bottom
        append UTF8('▀')

append reset
return RenderFrame(generation, index, out)
```

No newline. No direct stdout.

## 5. Exact RGB frame reader

```text
frame_size = checked(width * height * 3)
loop:
    buffer = vec(frame_size)
    n = read until frame_size or EOF
    if n == 0 at frame boundary:
        wait/check child status
        return clean EOF if child success else child error
    if 0 < n < frame_size:
        return truncated-frame error
    send DecodedFrame(index, buffer) into capacity-3 channel
    index += 1
```

Cancellation must race/interleave with reads through the chosen safe process/task design; do not leave child alive.

## 6. Latest-frame display model

```text
active_generation = none
current_status = none

loop:
    wait for either CONTROL or LATEST_FRAME_CHANGED, preferring CONTROL

    if CONTROL BeginGeneration(G, geometry):
        active_generation = G
        clear screen/viewport once
        reset styles

    if CONTROL Status(gen?, text):
        if gen is none or gen == active_generation:
            current_status = text
            draw/clear last row safely

    if FRAME_CHANGED:
        frame = newest frame
        if frame.generation != active_generation: discard
        else if suspended: discard
        else:
            write_all(frame.bytes)
            draw current status if policy requires
            flush

    if CONTROL Shutdown:
        reset + flush
        exit
```

Frames are latest-value. Do not queue seconds of ANSI data.

## 7. Next/Previous

```text
previous():
    if back empty: no-op
    forward.push_nearest(current)
    current = back.pop_nearest()
    enforce_history_bound()

next():
    if forward not empty:
        back.push_nearest(current)
        current = forward.pop_nearest()
    else:
        back.push_nearest(current)
        current = take(unseen_ready) or begin buffering next unseen
        forward.clear()
    enforce_history_bound()
```

`unseen_ready` is never reused as forward history storage.

## 8. Audio-master scheduling

```text
loop until cancelled/end:
    pos = audio_player_position()
    target = floor(pos_seconds * FPS)

    while decoded newest index < target and more decoded frames immediately available:
        consume/drop older decoded frames

    choose newest decoded frame appropriate for target
    if frame index newer than last presented:
        render/publish latest frame

    wait briefly for clock advance/frame/cancel; do not busy spin
```

Never stop audio just to preserve every video frame.
