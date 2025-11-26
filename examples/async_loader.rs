use egui_elm::prelude::*;
use std::time::Duration;

struct AsyncApp {
    data: Option<String>,
    loading: bool,
    request_count: u32,
}

#[derive(Clone)]
enum Message {
    Load,
    DataLoaded(String),
}

fn init() -> (AsyncApp, Command<Message>) {
    (
        AsyncApp {
            data: None,
            loading: false,
            request_count: 0,
        },
        Command::none(),
    )
}

fn update(model: &mut AsyncApp, message: Message) -> Command<Message> {
    match message {
        Message::Load => {
            if model.loading {
                return Command::none();
            }
            model.loading = true;
            let request_id = model.request_count + 1;
            Command::async_(async move {
                tokio::time::sleep(Duration::from_millis(8000)).await;
                Message::DataLoaded(format!("Async request #{request_id} complete"))
            })
        }
        Message::DataLoaded(payload) => {
            model.loading = false;
            model.request_count += 1;
            model.data = Some(payload);
            Command::none()
        }
    }
}

fn view(model: &AsyncApp, ctx: &egui::Context, ui_ctx: &ViewContext<Message>) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Async Loading Example");
        ui.label(format!("Request count: {}", model.request_count));

        if let Some(data) = &model.data {
            ui.colored_label(egui::Color32::LIGHT_GREEN, data);
        } else {
            ui.label("No data yet");
        }

        let button = ui.add_enabled(!model.loading, egui::Button::new("Load data"));
        if button.clicked() {
            ui_ctx.send(Message::Load);
        }

        if model.loading {
            ui.separator();
            ui.label("Loading, please wait...");
            ui.add(egui::Spinner::new());
        }
    });
}

fn subscription(_model: &AsyncApp) -> Subscription<Message> {
    Subscription::none()
}

fn main() -> eframe::Result<()> {
    let program = Program::new(init, update, view, subscription);
    egui_elm::app::run(program, "Async Loader")
}
