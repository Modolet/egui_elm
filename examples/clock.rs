use egui_elm::prelude::*;
use std::time::{Duration, SystemTime};

struct ClockModel {
    now: SystemTime,
}

#[derive(Clone)]
enum Message {
    Tick(SystemTime),
}

fn init() -> (ClockModel, Command<Message>) {
    (
        ClockModel {
            now: SystemTime::now(),
        },
        Command::none(),
    )
}

fn update(model: &mut ClockModel, message: Message) -> Command<Message> {
    match message {
        Message::Tick(now) => model.now = now,
    }
    Command::none()
}

fn view(model: &ClockModel, ctx: &egui::Context, _ui_ctx: &ViewContext<Message>) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("System clock");
        if let Ok(datetime) = model.now.duration_since(SystemTime::UNIX_EPOCH) {
            ui.monospace(format!(
                "UNIX: {}.{:03}s",
                datetime.as_secs(),
                datetime.subsec_millis()
            ));
        }
        ui.label(format!("Local time: {:?}", chrono::Local::now()));
    });
}

fn subscription(_model: &ClockModel) -> Subscription<Message> {
    Subscription::interval(Duration::from_secs(1), Message::Tick(SystemTime::now()))
}

fn main() -> eframe::Result<()> {
    let program = Program::new(init, update, view, subscription);
    egui_elm::app::run(program, "Clock")
}
