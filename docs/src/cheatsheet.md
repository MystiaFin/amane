# Cheat Sheet

Every builder method and function on one page. Each row links to the page that explains it.

All names come from `amane`. To bring in everything at once:

```rust
use amane::*;
# fn main() {}
```

## App

[Your First Shell](first_shell.md) · [Multiple Windows and Monitors](windows.md)

| Code | Does |
|---|---|
| `App::new()` | starts building the shell |
| `.window(view)` | adds a layer window, `fn() -> LayerWindow` |
| `.window_per_monitor(view)` | adds a layer window on every monitor, `fn(&Monitor) -> LayerWindow` |
| `.normal_window("name", view)` | adds a normal app window, `fn() -> Window` |
| `.lock(view)` | sets the lock screen, `fn(&Monitor) -> LayerWindow` |
| `.ipc("name", handler)` | handles `amane ipc call name`, `fn(&[String]) -> String` |
| `.font("Inter")` | sets the default font family |
| `.run()` | starts the shell. Never returns. |

## LayerWindow

[Layer Windows](layer_windows.md)

| Code | Does |
|---|---|
| `LayerWindow::new().width(w).height(h)` | required first. A number in pixels, or `Full`. |
| `.anchor_vertical(Vertical::Top)` | `Top`, `Middle` (default), `Bottom` |
| `.anchor_horizontal(Horizontal::Left)` | `Left`, `Middle` (default), `Right` |
| `.margin(Margin { top, right, bottom, left })` | gap from anchored edges, in pixels (`i32`) |
| `.layer(Layer::Top)` | `Background`, `Bottom`, `Top`, `Overlay` (default) |
| `.space(Zone::Reserve)` | `Reserve`, `Respect` (default), `Ignore` |
| `.keyboard(Keyboard::OnDemand)` | `None` (default), `OnDemand`, `Exclusive` |
| `.on_key(handler)` | handles keys, `Fn(Key)` |
| `.visible(bool)` | hides or shows the window, which keeps running |
| `.input_region(vec![InputArea { x, y, width, height }])` | only these areas take the mouse |
| `.click_through()` | no part takes the mouse |
| `.namespace("bar")` | name for compositor rules, read once when the window opens |
| `.child(widget)` | the window's content |

## Window

[Multiple Windows and Monitors](windows.md)

| Code | Does |
|---|---|
| `Window::new()` | a normal app window |
| `.title("Settings")` | window title, `"amane"` by default |
| `.size(480.0, 320.0)` | size it opens at, 640 by 480 by default |
| `.resizable(true)` | lets the user resize it, `false` by default |
| `.on_key(handler)` | handles keys, `Fn(Key)` |
| `.child(widget)` | the window's content |
| `open_window("name", view)` | opens a normal window later. Does nothing if that name is open. |
| `close_window("name")` | closes it |
| `window_size()` | the real size of the window being drawn, `(f32, f32)` |

## Sizes

[Layout](layout.md)

| Code | Means |
|---|---|
| `100.0` | exactly 100 pixels |
| `Parent` | as big as the parent allows. Several `Parent` children share the free space evenly. |
| `Full` | a whole monitor side. Only for `LayerWindow` sizes and `.radius(Full)`. |

## Row, Column, Stack

[Layout](layout.md)

| Code | Does |
|---|---|
| `Row::new(children![a, b])` | side by side |
| `Column::new(children![a, b])` | top to bottom |
| `Stack::new(children![a, b])` | on top of each other, later ones drawn over earlier ones |
| `.width(size)`, `.height(size)` | optional. Measured from the children by default. |
| `.gap(12.0)` | space between children (`Row`, `Column`) |
| `.justify(SpaceBetween)` | along the row: `Start`, `Center`, `End`, `SpaceBetween`, `SpaceAround`, `SpaceEvenly` |
| `.align(Center)` | across the row: `Start`, `Center`, `End` |
| `children![...]` | boxes a list of mixed widgets |
| `Vec<Box<dyn Widget>>` | a list built in a loop, with `Box::new(widget)` |

## Rectangle

[Rectangle](rectangle.md)

| Code | Does |
|---|---|
| `Rectangle::new().width(w).height(h)` | required first. A number or `Parent`. |
| `.fill("#1e1e2e")` | a hex color: `#rgb`, `#rrggbb`, `#rrggbbaa` |
| `.fill(Color::rgb(30, 30, 46))` | a color. Also `Color::rgba`, `Color::BLACK`, `WHITE`, `RED`, `GREEN`, `BLUE`, `TRANSPARENT`. |
| `.fill(Gradient::linear(90.0, [(0.0, "#a"), (1.0, "#b")]))` | a linear gradient. The angle works like CSS. |
| `.fill(Gradient::radial([(0.0, "#a"), (1.0, "#b")]))` | a radial gradient |
| `.fill(Image::cover(path))` | an image. Also `contain` and `stretch`. |
| `.fill(Mask)` | cuts a hole in the rectangle around it |
| `.radius(12.0)` | rounded corners |
| `.radius(Full)` | a pill or circle |
| `.radius_top_left(Full)` | one corner; also `radius_top_right`, `radius_bottom_right`, `radius_bottom_left`. Unset corners stay square. Panics if mixed with `.radius()`. |
| `.border(2.0, "#cdd6f4")` | an outline along the inside edge |
| `.opacity(0.5)` | fades it and its child |
| `.shadow(Shadow::drop("#000").blur(12.0).offset(0.0, 4.0).opacity(0.3))` | a drop shadow |
| `.shadow(Shadow::inner("#000").blur(8.0))` | an inner shadow |
| `.blur(20.0)` | blurs what your window drew behind it |
| `.padding(8.0)` | space inside the edges. Or `Padding { top, right, bottom, left }`. |
| `.align_child(Center, Center)` | places the child: horizontal, then vertical |
| `.clip()` | cuts the child off at the edges and corners |
| `.child(widget)` | one child |
| `.rotate(30.0)` | degrees, clockwise, around the center |
| `.scale(0.5)` | around the center |
| `.translate(20.0, -10.0)` | moves it without changing the layout |
| `.shader(path)` | a WGSL or GLSL shader over the fill |
| `.shader_values(vec![[a, b, c, d]])` | up to 16 rows of numbers for the shader |

## Input

[Input](input.md) · all on `Rectangle`

| Code | Called with |
|---|---|
| `.on_click(\|button\| ...)` | `Button::Left`, `Right`, `Middle` |
| `.on_hover(\|inside\| ...)` | `true` on enter, `false` on leave |
| `.on_scroll(\|scroll\| ...)` | `Scroll { x, y }` in lines, positive `y` is down |
| `.on_move(\|point\| ...)` | `Point { x, y }` from the top-left corner |
| `.on_drag(\|point\| ...)` | `Point`, from a left press until release |
| `.cursor(Pointer)` | `Default`, `Pointer`, `Text`, `Grab`, `Grabbing`, `Move`, `NotAllowed`, `Wait`, `Crosshair`, `Resize...` |

## Text

[Text](text.md)

| Code | Does |
|---|---|
| `Text::new("hello")` | takes `&str`, `String`, or `format!(...)` |
| `.size(20.0)` | 16 by default |
| `.color(Color::WHITE)` | takes a `Color`. Wrap hex in `Color::from("#...")`. |
| `.font("JetBrains Mono")` | a font family |
| `.weight(Weight::Bold)` | or a number like `300` |
| `.elide()` | one line, cut off with "…" |
| `.wrap()` | as many lines as needed |
| `.max_lines(2)` | with `wrap` |
| `.tight()` | measures the letters' shapes, for icon fonts |
| `.height_in(width)` | the height of wrapped text at that width |

## TextInput

[Text Input](text_input.md)

| Code | Does |
|---|---|
| `TextInput::new("id")` | the name its text is kept under |
| `.placeholder("search")` | shown while empty |
| `.password()` | shows dots |
| `.focused()` | takes keys without a click |
| `.on_change(\|text\| ...)` | after every change |
| `.on_submit(\|text\| ...)` | on Enter |
| `.size(18.0)`, `.color(c)`, `.width(w)` | look and size |
| `TextInput::set_text("id", "...")` | replaces its text |

## ScrollArea

[Scroll Area](scroll_area.md)

| Code | Does |
|---|---|
| `ScrollArea::new("id", child)` | scrolls `child` vertically. The name keeps its position. |
| `.width(size)`, `.height(size)` | `Parent` by default |

## Canvas and shapes

[Canvas and Shapes](canvas.md) · needs `Shape` in the `use` line

| Code | Does |
|---|---|
| `Canvas::new().width(w).height(h).shapes(shapes![...])` | draws shapes in its own coordinates |
| `Circle::new().center(x, y).radius(r)` | middle of the canvas, as big as fits, by default |
| `Arc::new().center(x, y).radius(r).start(deg).sweep(deg)` | 0 is up, clockwise |
| `Line::new().from(x, y).to(x, y)` | a straight line |
| `Path::new().move_to(x, y).line_to(x, y)` | also `quad_to`, `cubic_to`, `arc`, `close` |
| `.fill(color)` | paints the inside |
| `.stroke(4.0, color)` | draws the outline |
| `.cap(Cap::Round)` | `Butt` (default), `Round`, `Square` |
| `.opacity(0.5)` | fades the shape |

## Image

[Images](images.md)

| Code | Does |
|---|---|
| `Image::cover(path)` | fills, cuts off the edges |
| `Image::contain(path)` | shows all of it |
| `Image::stretch(path)` | fills, squashed to fit |
| `.thumbnail(w, h)` | keeps a small copy |
| `.blurred(radius)` | blurs it once, when loaded |
| `Image::loaded(path)` | `true` once it's ready to draw |

## Animation

[Animation](animation.md)

| Code | Does |
|---|---|
| `Animation::new(0.0)` | an animated `f32`. `Animation<Color>` for colors. |
| `.duration(Duration::from_millis(400))` | 200 ms by default |
| `.easing(Easing::InOut)` | `Out` (default), `InOut`, `Linear` |
| `animation.to(target)` | starts moving from wherever it is now |
| `animation.value()` | the current value. Keeps frames coming while it moves. |
| `amane::request_frame()` | asks for one more frame |

## Services

[State and Services](services.md)

| Code | Does |
|---|---|
| `impl Service for T { fn new() -> Self { ... } }` | the only required function |
| `fn interval() -> Duration` | how often `update` runs, 1 second by default |
| `fn update(&mut self) -> bool` | polls. Return `true` when something changed. |
| `fn listen()` | replaces polling with your own loop |
| `T::read()` | reads it. In a view, also subscribes the window. |
| `T::write()` | changes it, and redraws every window that reads it. Never in a view. |

## Built-in Services

[Built-in Services](system.md)

| Service | Reads | Controls |
|---|---|---|
| `Battery` | `present`, `percent`, `charging`, `full` | |
| `Cpu` | `percent` | |
| `Memory` | `percent`, `used_kib`, `total_kib` | |
| `Brightness` | `present`, `percent` | `set` |
| `Audio` | `volume`, `muted`, `microphone_volume`, `microphone_muted` | `set_volume`, `toggle_mute`, `set_microphone_volume`, `toggle_microphone_mute` |
| `Network` | `connected`, `link`, `ssid`, `strength`, `wifi_enabled`, `connecting`, `access_points` | `scan`, `connect`, `disconnect`, `set_wifi` |
| `Bluetooth` | `available`, `powered`, `scanning`, `devices` | `set_powered`, `start_scan`, `stop_scan`, `pair`, `connect`, `disconnect`, `forget` |
| `Media` | `players`, `active`, `title`, `artist`, `art_url`, `playing`, `position`, `length` | `play_pause`, `next`, `previous` |
| `Notifications` | `list`, `running` | `click`, `invoke`, `dismiss`, `clear` |
| `Workspaces` | `list` | `focus` |
| `Apps` | `list` | `launch` on each `DesktopApp` |
| `Palette` | `colors`, `dominant`, `accent`, `background`, `foreground`, `on_accent`, `light` | `open` |
| [`Pam`](pam.md) | `active`, `messages`, `prompt`, `result` | `start`, `respond`, `abort` |
| [`Lock`](lock_screen.md) | `checking`, `failed` | `start`, `unlock`, `authenticate`, `abort` |

## Commands, files, IPC, D-Bus

[Commands and Files](processes.md) · [IPC](ipc.md) · [D-Bus](dbus.md)

| Code | Does |
|---|---|
| `amane::spawn("cmd")` | starts a command, doesn't wait |
| `amane::output("cmd")` | runs a command and returns its output. Waits. |
| `amane::lines("cmd")` | each line a long-running command prints |
| `amane::watch_file(path)` | one item per change to a file |
| `Bus::system()`, `Bus::session()` | a D-Bus connection |
| `bus.property(dest, path, iface, name)` | reads a property |
| `bus.call(dest, path, iface, method, &arguments![...])` | calls a method |
| `bus.signals(iface, name)` | each matching signal |

## Command line

[The amane Command](cli.md)

| Command | Does |
|---|---|
| `amane startup` | creates `~/.config/amane/src/main.rs` |
| `amane dev` | rebuilds and restarts on every save |
| `amane compile` | builds the shell `amane run` starts |
| `amane run` | starts the compiled shell, never builds |
| `amane clean` | deletes the build output, keeps the compiled shell, asks first |
| `amane clean --yes` | the same, without asking |
| `amane ipc call <name> [args...]` | calls an IPC handler |
| `AMANE_FRAMES=1` | prints a line per frame, for finding slow or endless redraws |
