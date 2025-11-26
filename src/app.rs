use std::sync::mpsc;

use eframe::egui;
use futures::StreamExt;
use tokio::{runtime::Runtime, task::JoinHandle};

use crate::{command::Command, program::Program, subscription::Subscription, view::ViewContext};

/// Runs the supplied Elm program using eframe's native runner.
pub fn run<Model, Message>(program: Program<Model, Message>, title: &str) -> eframe::Result<()>
where
    Model: Send + 'static,
    Message: Send + 'static,
{
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        title,
        native_options,
        Box::new(move |_cc| {
            let runtime = Runtime::new()?;
            let (model, command) = (program.init)();
            let app: Box<dyn eframe::App> = Box::new(ElmApp::new(program, model, command, runtime));
            Ok(app)
        }),
    )
}

struct ElmApp<Model, Message>
where
    Model: Send + 'static,
    Message: Send + 'static,
{
    program: Program<Model, Message>,
    model: Model,
    runtime: Runtime,
    mailbox_sender: mpsc::Sender<Message>,
    mailbox_receiver: mpsc::Receiver<Message>,
    subscription_task: Option<JoinHandle<()>>,
}

impl<Model, Message> ElmApp<Model, Message>
where
    Model: Send + 'static,
    Message: Send + 'static,
{
    fn new(
        program: Program<Model, Message>,
        model: Model,
        initial_command: Command<Message>,
        runtime: Runtime,
    ) -> Self {
        let (mailbox_sender, mailbox_receiver) = mpsc::channel();

        let mut app = Self {
            program,
            model,
            runtime,
            mailbox_sender: mailbox_sender.clone(),
            mailbox_receiver,
            subscription_task: None,
        };

        app.enqueue_command(initial_command);
        app.restart_subscription();
        app
    }

    fn enqueue_command(&mut self, command: Command<Message>) {
        for future in command.into_futures() {
            let sender = self.mailbox_sender.clone();
            self.runtime.spawn(async move {
                if let Some(message) = future.await {
                    let _ = sender.send(message);
                }
            });
        }
    }

    fn spawn_subscription(
        runtime: &Runtime,
        subscription: Subscription<Message>,
        sender: mpsc::Sender<Message>,
    ) -> JoinHandle<()> {
        runtime.spawn(async move {
            let mut stream = subscription.into_stream();
            while let Some(message) = stream.next().await {
                if sender.send(message).is_err() {
                    break;
                }
            }
        })
    }

    fn restart_subscription(&mut self) {
        if let Some(handle) = self.subscription_task.take() {
            handle.abort();
        }

        let subscription = (self.program.subscription)(&self.model);
        self.subscription_task = Some(Self::spawn_subscription(
            &self.runtime,
            subscription,
            self.mailbox_sender.clone(),
        ));
    }

    fn handle_message(&mut self, message: Message) {
        let command = (self.program.update)(&mut self.model, message);
        self.enqueue_command(command);
        self.restart_subscription();
    }
}

impl<Model, Message> Drop for ElmApp<Model, Message>
where
    Model: Send + 'static,
    Message: Send + 'static,
{
    fn drop(&mut self) {
        if let Some(handle) = self.subscription_task.take() {
            handle.abort();
        }
    }
}

impl<Model, Message> eframe::App for ElmApp<Model, Message>
where
    Model: Send + 'static,
    Message: Send + 'static,
{
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(message) = self.mailbox_receiver.try_recv() {
            self.handle_message(message);
        }

        let view_context = ViewContext::new(self.mailbox_sender.clone());
        (self.program.view)(&self.model, ctx, &view_context);

        ctx.request_repaint();
    }
}
