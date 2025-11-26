use std::future::Future;

use futures::{future::BoxFuture, FutureExt};

/// Represents asynchronous work to be performed by the Elm runtime.
pub struct Command<Message>
where
    Message: Send + 'static,
{
    tasks: Vec<CommandFuture<Message>>,
}

/// Boxed future used internally by commands.
pub type CommandFuture<Message> = BoxFuture<'static, Option<Message>>;

impl<Message> Command<Message>
where
    Message: Send + 'static,
{
    /// Creates a command that performs no work.
    pub fn none() -> Self {
        Self { tasks: Vec::new() }
    }

    /// Creates a command that immediately produces the provided message.
    pub fn message(message: Message) -> Self {
        Self::from_optional_future(async move { Some(message) })
    }

    /// Creates a command from a future that yields a message.
    pub fn async_<Fut>(future: Fut) -> Self
    where
        Fut: Future<Output = Message> + Send + 'static,
    {
        Self::from_optional_future(async move { Some(future.await) })
    }

    /// Creates a command from a future that may yield a message.
    pub fn from_optional_future<Fut>(future: Fut) -> Self
    where
        Fut: Future<Output = Option<Message>> + Send + 'static,
    {
        Self {
            tasks: vec![future.boxed()],
        }
    }

    /// Creates a command from a synchronous computation.
    pub fn perform<F>(op: F) -> Self
    where
        F: FnOnce() -> Message + Send + 'static,
    {
        Self::from_optional_future(async move { Some(op()) })
    }

    /// Batches multiple commands together so they can run in parallel.
    pub fn batch(commands: impl IntoIterator<Item = Self>) -> Self {
        let tasks = commands
            .into_iter()
            .flat_map(|command| command.tasks)
            .collect();

        Self { tasks }
    }

    /// Transforms the message type produced by the command.
    pub fn map<F, Output>(self, f: F) -> Command<Output>
    where
        Output: Send + 'static,
        F: Fn(Message) -> Output + Send + Sync + 'static,
    {
        let f = std::sync::Arc::new(f);
        let tasks = self
            .tasks
            .into_iter()
            .map(|task| {
                let f = f.clone();
                task.map(move |maybe_message| maybe_message.map(|message| f(message)))
                    .boxed()
            })
            .collect();

        Command { tasks }
    }

    #[cfg_attr(not(feature = "runtime"), allow(dead_code))]
    pub(crate) fn into_futures(self) -> Vec<CommandFuture<Message>> {
        self.tasks
    }
}

impl<Message> Default for Command<Message>
where
    Message: Send + 'static,
{
    fn default() -> Self {
        Self::none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;

    #[test]
    fn message_command_completes() {
        let mut futures = Command::message(5).into_futures();
        let output = block_on(futures.pop().expect("future")).expect("message");
        assert_eq!(output, 5);
    }

    #[test]
    fn batch_flattens_all_commands() {
        let a = Command::message("a");
        let b = Command::message("b");
        let combined = Command::batch([a, b]);
        let mut futures = combined.into_futures();

        let mut results = Vec::new();
        for future in futures.drain(..) {
            results.push(block_on(future).unwrap());
        }
        results.sort();
        assert_eq!(results, vec!["a", "b"]);
    }
}
