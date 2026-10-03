# Troubleshooting

## The shell won't start

| Message | Cause | Fix |
|---|---|---|
| `failed to bind wlr-layer-shell` | your compositor doesn't support layer windows | use a compositor that does, like niri, Hyprland, Sway, or river |
| `failed to listen: amane is already running` | another Amane shell is running in this session | stop it first, see [IPC](ipc.md) |
| `failed to run: no window set` | `App` was given no windows | add at least one `.window`, `.window_per_monitor`, `.normal_window`, or `.lock` |
| `failed to read shader` | a shader path is wrong | use an absolute path, see [Shaders](shaders.md) |

## The window looks wrong

| Problem | Fix |
|---|---|
| Normal windows cover the bar | add `.space(Zone::Reserve)` to the bar ([Layer Windows](layer_windows.md)) |
| The bar covers fullscreen videos | add `.layer(Layer::Top)`. The default is `Overlay`, which is above everything. |
| The window is in the middle of the screen | add `anchor_vertical` or `anchor_horizontal`. Both default to `Middle`. |
| A color shows bright pink | that hex string is mistyped. Amane shows pink instead of crashing. |
| An image doesn't show | check the path is absolute and the file is PNG, JPEG, or SVG ([Images](images.md)) |
| Text runs past its box | add `.elide()` or `.wrap()` ([Text](text.md)) |
| Shape methods like `.stroke` don't exist | add `Shape` to your `use` line ([Canvas and Shapes](canvas.md)) |

## Input doesn't work

| Problem | Fix |
|---|---|
| Typing does nothing | give the window `.keyboard(Keyboard::OnDemand)` or `Keyboard::Exclusive` ([Layer Windows](layer_windows.md)) |
| `on_key` never runs | same as above. Keys only reach a window that has keyboard focus. |
| Clicks go through the window | check for `.click_through()` or an `.input_region(...)` that leaves that part out |
| Clicks don't go through an empty part of the window | give the window an `.input_region(...)`, or hide it with `.visible(false)` while it's empty |

## The shell freezes

- **A view calls `write()`.** A view may still hold a `read()` of the same Service, and the write waits for it forever. Only read in views ([State and Services](services.md)).
- **A view waits on something.** `amane::output`, file reads, D-Bus calls, and `sleep` all block the thread that draws every window. Move them into a Service.

## The shell uses too much CPU

Run your shell with `AMANE_FRAMES=1` to print a line for every frame:

```sh
AMANE_FRAMES=1 ~/.cache/amane/project/target/release/amane-shell
```

Each line shows which window drew, the time since its last frame, and how long the view, the drawing, and the GPU took. It also prints which Service woke the windows. Look for:

- **Frames that never stop.** Something keeps asking for frames: a view that always calls `request_frame()`, or a shader whose source contains the word `time` ([Shaders](shaders.md)).
- **A Service that wakes windows every poll.** Its `update()` returns `true` even when nothing changed. Compare the old and new values, and return whether they differ.
- **A slow view.** Something in the view is doing real work. Move it into a Service.

## The lock screen won't unlock

See [Testing safely](lock_screen.md#testing-safely) for how to get back into a locked session.
