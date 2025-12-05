//! egui_elm: Elm-style architecture for egui applications.

#[cfg(feature = "runtime")]
pub mod app;
pub mod command;
pub mod program;
pub mod subscription;
pub mod view;

pub mod prelude {
    #[cfg(feature = "runtime")]
    pub use crate::app::{run, run_with_native_options};
    pub use crate::{
        command::Command,
        program::Program,
        subscription::{IntoSubscription, StreamSubscription, Subscription, SubscriptionToken},
        view::ViewContext,
    };
}
