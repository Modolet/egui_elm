use crate::{
    command::Command,
    subscription::{IntoSubscription, Subscription},
    view::ViewFn,
};

#[cfg(feature = "runtime")]
type SaveHandler<Model> = fn(&mut Model, &mut dyn eframe::Storage);

#[cfg(feature = "runtime")]
type ExitHandler<Model> = fn(&mut Model, Option<&glow::Context>);

/// Describes the four pure functions that make up an Elm-style program.
pub struct Program<Model, Message, Sub = Subscription<Message>>
where
    Model: Send + 'static,
    Message: Send + 'static,
    Sub: IntoSubscription<Message> + Send + 'static,
{
    pub init: fn(&egui::Context) -> (Model, Command<Message>),
    pub update: fn(&mut Model, Message) -> Command<Message>,
    pub view: ViewFn<Model, Message>,
    pub subscription: fn(&Model) -> Sub,
    #[cfg(feature = "runtime")]
    pub(crate) save: Option<SaveHandler<Model>>,
    #[cfg(feature = "runtime")]
    pub(crate) on_exit: Option<ExitHandler<Model>>,
}

impl<Model, Message, Sub> Program<Model, Message, Sub>
where
    Model: Send + 'static,
    Message: Send + 'static,
    Sub: IntoSubscription<Message> + Send + 'static,
{
    /// Creates a new [Program](crate::program::Program) by wiring together the init/update/view/subscription functions.
    pub fn new(
        init: fn(&egui::Context) -> (Model, Command<Message>),
        update: fn(&mut Model, Message) -> Command<Message>,
        view: ViewFn<Model, Message>,
        subscription: fn(&Model) -> Sub,
    ) -> Self {
        Self {
            init,
            update,
            view,
            subscription,
            #[cfg(feature = "runtime")]
            save: None,
            #[cfg(feature = "runtime")]
            on_exit: None,
        }
    }
}

#[cfg(feature = "runtime")]
impl<Model, Message, Sub> Program<Model, Message, Sub>
where
    Model: Send + 'static,
    Message: Send + 'static,
    Sub: IntoSubscription<Message> + Send + 'static,
{
    /// Registers a callback that will be invoked from eframe's [`App::save`](eframe::App::save).
    pub fn with_save(mut self, save: SaveHandler<Model>) -> Self {
        self.save = Some(save);
        self
    }

    /// Registers a callback that will be invoked from eframe's [`App::on_exit`](eframe::App::on_exit).
    pub fn with_on_exit(mut self, on_exit: ExitHandler<Model>) -> Self {
        self.on_exit = Some(on_exit);
        self
    }
}
