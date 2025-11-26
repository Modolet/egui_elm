use egui_elm::prelude::*;
use std::time::Duration;

#[derive(Default)]
struct Counter {
    count: i32,
    auto: bool,
}

#[derive(Clone)]
enum Message {
    Increment,
    Decrement,
    ToggleAuto(bool),
    Tick,
}

fn init() -> (Counter, Command<Message>) {
    (Counter::default(), Command::none())
}

fn update(model: &mut Counter, message: Message) -> Command<Message> {
    match message {
        Message::Increment => {
            model.count += 1;
        }
        Message::Decrement => {
            model.count -= 1;
        }
        Message::ToggleAuto(on) => {
            model.auto = on;
        }
        Message::Tick => {
            if model.auto {
                model.count += 1;
            }
        }
    }
    Command::none()
}

fn view(model: &Counter, ctx: &egui::Context, ui_ctx: &ViewContext<Message>) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Counter");
        ui.label(format!("Count: {}", model.count));

        ui.horizontal(|ui| {
            if ui.button("+").clicked() {
                ui_ctx.send(Message::Increment);
            }
            if ui.button("-").clicked() {
                ui_ctx.send(Message::Decrement);
            }
        });

        let mut auto = model.auto;
        if ui.checkbox(&mut auto, "Auto increment").clicked() {
            ui_ctx.send(Message::ToggleAuto(auto));
        }
    });
}

fn subscription(model: &Counter) -> Subscription<Message> {
    if model.auto {
        Subscription::interval(Duration::from_secs(1), Message::Tick)
    } else {
        Subscription::none()
    }
}

fn main() -> eframe::Result<()> {
    let program = Program::new(init, update, view, subscription);
    egui_elm::app::run(program, "Counter")
}
