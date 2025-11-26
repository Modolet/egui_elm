use crate::{command::Command, subscription::Subscription, view::ViewFn};

/// Describes the four pure functions that make up an Elm-style program.
pub struct Program<Model, Message>
where
    Model: Send + 'static,
    Message: Send + 'static,
{
    pub init: fn() -> (Model, Command<Message>),
    pub update: fn(&mut Model, Message) -> Command<Message>,
    pub view: ViewFn<Model, Message>,
    pub subscription: fn(&Model) -> Subscription<Message>,
}

impl<Model, Message> Program<Model, Message>
where
    Model: Send + 'static,
    Message: Send + 'static,
{
    /// Creates a new [`Program`] by wiring together the init/update/view/subscription functions.
    pub fn new(
        init: fn() -> (Model, Command<Message>),
        update: fn(&mut Model, Message) -> Command<Message>,
        view: ViewFn<Model, Message>,
        subscription: fn(&Model) -> Subscription<Message>,
    ) -> Self {
        Self {
            init,
            update,
            view,
            subscription,
        }
    }
}
