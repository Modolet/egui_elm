# egui_elm

`egui_elm` brings an Elm-style, purely functional architecture to [`egui`](https://github.com/emilk/egui). It focuses on a simple `init / update / view / subscription` model so you can write declarative desktop GUIs with predictable state transitions and explicit commands.

## Features

- Elm-inspired `Program` structure with standalone functions instead of traits.
- Explicit `Command` type for asynchronous/side-effectful work.
- `Subscription` abstraction for continuous external events such as timers.
- Optional `app` runtime (`run`) built on top of `eframe` + `tokio`. Disable the default `runtime` feature if you only need the architectural pieces.

## Getting started

```toml
[dependencies]
egui_elm = "0.3.3"
```

By default the crate enables the `runtime` feature, which pulls in the native runner and Tokio. To depend on only the core types, disable default features:

```toml
[dependencies]
egui_elm = { version = "0.3.3", default-features = false }
```

## Quick example

```rust
use egui_elm::prelude::*;

#[derive(Default)]
struct Counter {
    value: i32,
}

#[derive(Clone)]
enum Message {
    Increment,
    Decrement,
}

fn init(_ctx: &egui::Context) -> (Counter, Command<Message>) {
    (Counter::default(), Command::none())
}

fn update(model: &mut Counter, message: Message) -> Command<Message> {
    match message {
        Message::Increment => model.value += 1,
        Message::Decrement => model.value -= 1,
    }
    Command::none()
}

fn view(model: &Counter, ctx: &egui::Context, ui_ctx: &ViewContext<Message>) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Counter");
        ui.label(format!("Value: {}", model.value));
        if ui.button("Increment").clicked() {
            ui_ctx.send(Message::Increment);
        }
        if ui.button("Decrement").clicked() {
            ui_ctx.send(Message::Decrement);
        }
    });
}

fn subscription(_model: &Counter) -> Subscription<Message> {
    Subscription::none()
}

fn main() -> eframe::Result<()> {
    let program = Program::new(init, update, view, subscription);
    egui_elm::app::run(program, "Counter")
}
```

More runnable examples live in [`examples/`](examples/).

### Selecting a renderer

`egui_elm::app::run` uses the default `eframe::NativeOptions`. If you need to force a specific backend
such as `wgpu` or tweak any other option, call `run_with_native_options` instead. Enable the crate's
`wgpu` feature so that `eframe` also builds with its `wgpu` renderer:

```toml
[dependencies]
egui_elm = { version = "0.3.3", features = ["wgpu"] }
```

```rust
fn main() -> eframe::Result<()> {
    let program = Program::new(init, update, view, subscription);
    let mut native_options = eframe::NativeOptions::default();
    native_options.renderer = eframe::Renderer::Wgpu;
    egui_elm::app::run_with_native_options(program, "Counter", native_options)
}
```

### eframe hooks

When the `runtime` feature is enabled you can bridge into eframe's `save` and `on_exit` lifecycle callbacks with `Program::with_save` and `Program::with_on_exit`:

```rust
fn save(model: &mut Counter, storage: &mut dyn eframe::Storage) {
    storage.set_string("count", model.value.to_string());
}

fn on_exit(model: &mut Counter, _gl: Option<&glow::Context>) {
    eprintln!("Goodbye with count = {}", model.value);
}

fn main() -> eframe::Result<()> {
    let program = Program::new(init, update, view, subscription)
        .with_save(save)
        .with_on_exit(on_exit);
    egui_elm::app::run(program, "Counter")
}
```

## License

MIT. See [LICENSE](LICENSE) for details.
