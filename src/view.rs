use egui::Context;

/// Context passed down to view functions so they can send messages back to the update loop.
#[derive(Clone)]
pub struct ViewContext<Message>
where
    Message: Send + 'static,
{
    sender: std::sync::mpsc::Sender<Message>,
}

impl<Message> ViewContext<Message>
where
    Message: Send + 'static,
{
    #[cfg_attr(not(feature = "runtime"), allow(dead_code))]
    pub(crate) fn new(sender: std::sync::mpsc::Sender<Message>) -> Self {
        Self { sender }
    }

    /// Sends a message back to the Elm program without blocking the UI thread.
    pub fn send(&self, message: Message) {
        let _ = self.sender.send(message);
    }
}

/// Shared type alias for view functions used by [`Program`](crate::program::Program).
pub type ViewFn<Model, Message> = fn(&Model, &Context, &ViewContext<Message>);
