use egui::Context;

#[cfg(feature = "runtime")]
type ViewSender<Message> = tokio::sync::mpsc::Sender<Message>;

#[cfg(not(feature = "runtime"))]
type ViewSender<Message> = std::sync::mpsc::Sender<Message>;

/// Context passed down to view functions so they can send messages back to the update loop.
#[derive(Clone)]
pub struct ViewContext<Message>
where
    Message: Send + 'static,
{
    sender: ViewSender<Message>,
}

impl<Message> ViewContext<Message>
where
    Message: Send + 'static,
{
    #[cfg_attr(not(feature = "runtime"), allow(dead_code))]
    pub(crate) fn new(sender: ViewSender<Message>) -> Self {
        Self { sender }
    }
}

#[cfg(feature = "runtime")]
impl<Message> ViewContext<Message>
where
    Message: Send + 'static,
{
    /// Sends a message back to the Elm program without blocking the UI thread.
    pub fn send(&self, message: Message) {
        let _ = self.sender.try_send(message);
    }
}

#[cfg(not(feature = "runtime"))]
impl<Message> ViewContext<Message>
where
    Message: Send + 'static,
{
    /// Sends a message back to the Elm program without blocking the UI thread.
    pub fn send(&self, message: Message) {
        let _ = self.sender.send(message);
    }
}

/// Shared type alias for view functions used by [`Program`](crate::program::Program).
pub type ViewFn<Model, Message> = fn(&Model, &Context, &ViewContext<Message>);
