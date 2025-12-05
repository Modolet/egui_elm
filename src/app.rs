use std::future::Future;

use eframe::egui;
use futures::{pin_mut, Stream, StreamExt};
use tokio::{
    runtime::{Handle, Runtime},
    sync::mpsc,
    task::JoinHandle,
};

use crate::{
    command::Command,
    program::Program,
    subscription::{IntoSubscription, SubscriptionToken},
    view::ViewContext,
};

const MAILBOX_CAPACITY: usize = 512;

/// Runs the supplied Elm program using eframe's native runner with default options.
///
/// To customize the renderer (e.g. switch between `glow` and `wgpu`) or any other
/// [`eframe::NativeOptions`], call [`run_with_native_options`].
pub fn run<Model, Message, Sub>(
    program: Program<Model, Message, Sub>,
    title: &str,
) -> eframe::Result<()>
where
    Model: Send + 'static,
    Message: Send + 'static,
    Sub: IntoSubscription<Message> + Send + 'static,
{
    run_with_native_options(program, title, eframe::NativeOptions::default())
}

/// Runs the supplied Elm program using eframe's native runner and custom options.
pub fn run_with_native_options<Model, Message, Sub>(
    program: Program<Model, Message, Sub>,
    title: &str,
    native_options: eframe::NativeOptions,
) -> eframe::Result<()>
where
    Model: Send + 'static,
    Message: Send + 'static,
    Sub: IntoSubscription<Message> + Send + 'static,
{
    eframe::run_native(
        title,
        native_options,
        Box::new(move |cc| {
            let runtime = TokioRuntime::try_current_or_new()?;
            let (model, command) = (program.init)(&cc.egui_ctx);
            let app: Box<dyn eframe::App> = Box::new(ElmApp::new(program, model, command, runtime));
            Ok(app)
        }),
    )
}

enum TokioRuntime {
    Owned(Runtime),
    Handle(Handle),
}

impl TokioRuntime {
    fn try_current_or_new() -> std::io::Result<Self> {
        match Handle::try_current() {
            Ok(handle) => Ok(Self::Handle(handle)),
            Err(_) => Runtime::new().map(Self::Owned),
        }
    }

    fn spawn<F>(&self, future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        match self {
            Self::Owned(runtime) => runtime.spawn(future),
            Self::Handle(handle) => handle.spawn(future),
        }
    }
}

struct ElmApp<Model, Message, Sub>
where
    Model: Send + 'static,
    Message: Send + 'static,
    Sub: IntoSubscription<Message> + Send + 'static,
{
    program: Program<Model, Message, Sub>,
    model: Model,
    runtime: TokioRuntime,
    mailbox_sender: mpsc::Sender<Message>,
    mailbox_receiver: mpsc::Receiver<Message>,
    subscription_task: Option<JoinHandle<()>>,
    subscription_token: Option<SubscriptionToken>,
}

impl<Model, Message, Sub> ElmApp<Model, Message, Sub>
where
    Model: Send + 'static,
    Message: Send + 'static,
    Sub: IntoSubscription<Message> + Send + 'static,
{
    fn new(
        program: Program<Model, Message, Sub>,
        model: Model,
        initial_command: Command<Message>,
        runtime: TokioRuntime,
    ) -> Self {
        let (mailbox_sender, mailbox_receiver) = mpsc::channel(MAILBOX_CAPACITY);

        let mut app = Self {
            program,
            model,
            runtime,
            mailbox_sender: mailbox_sender.clone(),
            mailbox_receiver,
            subscription_task: None,
            subscription_token: None,
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
                    let _ = sender.send(message).await;
                }
            });
        }
    }

    fn spawn_stream<S>(
        runtime: &TokioRuntime,
        stream: S,
        sender: mpsc::Sender<Message>,
    ) -> JoinHandle<()>
    where
        S: Stream<Item = Message> + Send + 'static,
    {
        runtime.spawn(async move {
            pin_mut!(stream);
            while let Some(message) = stream.next().await {
                if sender.send(message).await.is_err() {
                    break;
                }
            }
        })
    }

    fn restart_subscription(&mut self) {
        let subscription = (self.program.subscription)(&self.model);
        let identity = subscription.identity();

        if let (Some(previous), Some(current)) = (&self.subscription_token, &identity) {
            if previous == current {
                return;
            }
        }

        if let Some(handle) = self.subscription_task.take() {
            handle.abort();
        }

        let stream = subscription.into_stream();
        self.subscription_task = Some(Self::spawn_stream(
            &self.runtime,
            stream,
            self.mailbox_sender.clone(),
        ));
        self.subscription_token = identity;
    }

    fn handle_message(&mut self, message: Message) {
        let command = (self.program.update)(&mut self.model, message);
        self.enqueue_command(command);
        self.restart_subscription();
    }
}

impl<Model, Message, Sub> Drop for ElmApp<Model, Message, Sub>
where
    Model: Send + 'static,
    Message: Send + 'static,
    Sub: IntoSubscription<Message> + Send + 'static,
{
    fn drop(&mut self) {
        if let Some(handle) = self.subscription_task.take() {
            handle.abort();
        }
    }
}

impl<Model, Message, Sub> eframe::App for ElmApp<Model, Message, Sub>
where
    Model: Send + 'static,
    Message: Send + 'static,
    Sub: IntoSubscription<Message> + Send + 'static,
{
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(message) = self.mailbox_receiver.try_recv() {
            self.handle_message(message);
        }

        let view_context = ViewContext::new(self.mailbox_sender.clone());
        (self.program.view)(&self.model, ctx, &view_context);

        ctx.request_repaint();
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if let Some(save) = self.program.save {
            save(&mut self.model, storage);
        }
    }

    fn on_exit(&mut self, gl: Option<&glow::Context>) {
        if let Some(on_exit) = self.program.on_exit {
            on_exit(&mut self.model, gl);
        }
    }
}
