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
egui_elm = "0.2"
```

By default the crate enables the `runtime` feature, which pulls in the native runner and Tokio. To depend on only the core types, disable default features:

```toml
[dependencies]
egui_elm = { version = "0.2", default-features = false }
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

fn init() -> (Counter, Command<Message>) {
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

## License

MIT. See [LICENSE](LICENSE) for details.
