use egui_elm::prelude::*;
use std::time::{Duration, Instant};

#[derive(Clone)]
enum Message {
    Tick,
    Toggle(bool),
    Reset,
}

struct TimerApp {
    running: bool,
    started_at: Instant,
    elapsed: Duration,
}

fn init() -> (TimerApp, Command<Message>) {
    (
        TimerApp {
            running: false,
            started_at: Instant::now(),
            elapsed: Duration::ZERO,
        },
        Command::none(),
    )
}

fn update(model: &mut TimerApp, message: Message) -> Command<Message> {
    match message {
        Message::Tick => {
            if model.running {
                model.elapsed = Instant::now() - model.started_at;
            }
        }
        Message::Toggle(on) => {
            model.running = on;
            model.started_at = Instant::now() - model.elapsed;
        }
        Message::Reset => {
            model.running = false;
            model.elapsed = Duration::ZERO;
        }
    }
    Command::none()
}

fn view(model: &TimerApp, ctx: &egui::Context, ui_ctx: &ViewContext<Message>) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Simple timer");
        ui.label(format!("Elapsed time: {:.2?}", model.elapsed));

        if ui
            .button(if model.running { "Pause" } else { "Start" })
            .clicked()
        {
            ui_ctx.send(Message::Toggle(!model.running));
        }

        if ui.button("Reset").clicked() {
            ui_ctx.send(Message::Reset);
        }
    });
}

fn subscription(model: &TimerApp) -> Subscription<Message> {
    if model.running {
        Subscription::interval(Duration::from_millis(50), Message::Tick)
    } else {
        Subscription::none()
    }
}

fn main() -> eframe::Result<()> {
    let program = Program::new(init, update, view, subscription);
    egui_elm::app::run(program, "Timer")
}
