use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use async_stream::stream;
use futures::{stream::SelectAll, Stream, StreamExt};
use futures_timer::Delay;

/// Represents a continuous stream of incoming messages for an Elm program.
pub struct Subscription<Message>
where
    Message: Send + 'static,
{
    stream: Pin<Box<dyn Stream<Item = Message> + Send>>,
}

impl<Message> Subscription<Message>
where
    Message: Send + 'static,
{
    /// Creates a subscription that yields no values.
    pub fn none() -> Self {
        Self {
            stream: Box::pin(futures::stream::pending()),
        }
    }

    /// Creates a subscription from any stream of messages.
    pub fn from_stream<S>(stream: S) -> Self
    where
        S: Stream<Item = Message> + Send + 'static,
    {
        Self {
            stream: Box::pin(stream),
        }
    }

    /// Batches multiple subscriptions into a single stream of messages.
    pub fn batch(subscriptions: impl IntoIterator<Item = Self>) -> Self {
        let mut select_all: SelectAll<_> = SelectAll::new();
        for subscription in subscriptions {
            select_all.push(subscription.stream);
        }

        Self {
            stream: Box::pin(select_all),
        }
    }

    /// Creates a subscription by periodically emitting a message.
    pub fn interval(duration: Duration, message: Message) -> Self
    where
        Message: Clone,
    {
        let stream = stream! {
            loop {
                Delay::new(duration).await;
                yield message.clone();
            }
        };

        Self::from_stream(stream)
    }

    /// Maps the output of the subscription into a different message type.
    pub fn map<F, Output>(self, f: F) -> Subscription<Output>
    where
        F: FnMut(Message) -> Output + Send + 'static,
        Output: Send + 'static,
    {
        let mapped_stream = self.stream.map(f);
        Subscription::from_stream(mapped_stream)
    }

    #[cfg_attr(not(feature = "runtime"), allow(dead_code))]
    pub(crate) fn into_stream(self) -> Pin<Box<dyn Stream<Item = Message> + Send>> {
        self.stream
    }
}

impl<Message> Default for Subscription<Message>
where
    Message: Send + 'static,
{
    fn default() -> Self {
        Self::none()
    }
}

impl<Message> Stream for Subscription<Message>
where
    Message: Send + 'static,
{
    type Item = Message;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.stream.as_mut().poll_next(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::{executor::block_on, StreamExt};
    use std::time::Duration;

    #[test]
    fn batch_merges_streams() {
        let combined = Subscription::batch(vec![
            Subscription::from_stream(futures::stream::iter(vec![1, 2])),
            Subscription::from_stream(futures::stream::iter(vec![3])),
        ]);

        let mut stream = combined.into_stream();
        let result = block_on(async {
            let mut values = Vec::new();
            while let Some(value) = stream.next().await {
                values.push(value);
                if values.len() == 3 {
                    break;
                }
            }
            values
        });

        assert_eq!(result.len(), 3);
        assert!(result.contains(&1));
        assert!(result.contains(&2));
        assert!(result.contains(&3));
    }

    #[test]
    fn map_transforms_messages() {
        let subscription = Subscription::from_stream(futures::stream::iter(vec![1, 2, 3]));
        let mut stream = subscription.map(|value| value * 2).into_stream();
        let doubled = block_on(async {
            let mut values = Vec::new();
            while let Some(value) = stream.next().await {
                values.push(value);
            }
            values
        });

        assert_eq!(doubled, vec![2, 4, 6]);
    }

    #[test]
    fn interval_emits_multiple_messages() {
        let subscription = Subscription::interval(Duration::from_millis(10), 42);
        let mut stream = subscription.into_stream();
        let values = block_on(async {
            let mut values = Vec::new();
            while let Some(value) = stream.next().await {
                values.push(value);
                if values.len() == 2 {
                    break;
                }
            }
            values
        });

        assert_eq!(values, vec![42, 42]);
    }
}
