use std::{
    any::Any,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};

use async_stream::stream;
use futures::{stream::SelectAll, Stream, StreamExt};
use futures_timer::Delay;

/// Trait implemented by values that can be converted into a subscription stream.
pub trait IntoSubscription<Message>: Send + 'static
where
    Message: Send + 'static,
{
    /// Concrete stream type produced by the subscription.
    type Stream: Stream<Item = Message> + Send + 'static;

    /// Returns an optional identity token for this subscription.
    fn identity(&self) -> Option<SubscriptionToken>;

    /// Consumes the subscription and returns the underlying stream.
    fn into_stream(self) -> Self::Stream;
}

/// Represents a continuous stream of incoming messages for an Elm program.
pub struct Subscription<Message>
where
    Message: Send + 'static,
{
    stream: Pin<Box<dyn Stream<Item = Message> + Send>>,
    token: Option<SubscriptionToken>,
}

impl<Message> Subscription<Message>
where
    Message: Send + 'static,
{
    /// Creates a subscription that yields no values.
    pub fn none() -> Self {
        Self {
            stream: Box::pin(futures::stream::pending()),
            token: Some(SubscriptionToken::new(())),
        }
    }

    /// Creates a subscription from any stream of messages.
    pub fn from_stream<S>(stream: S) -> Self
    where
        S: Stream<Item = Message> + Send + 'static,
    {
        Self {
            stream: Box::pin(stream),
            token: None,
        }
    }

    /// Batches multiple subscriptions into a single stream of messages.
    pub fn batch(subscriptions: impl IntoIterator<Item = Self>) -> Self {
        let mut select_all: SelectAll<_> = SelectAll::new();
        let mut tokens = Vec::new();
        let mut missing_identity = false;
        for subscription in subscriptions {
            match subscription.token {
                Some(token) => tokens.push(token),
                None => missing_identity = true,
            }
            select_all.push(subscription.stream);
        }

        Self {
            stream: Box::pin(select_all),
            token: if missing_identity {
                None
            } else {
                Some(SubscriptionToken::new(tokens))
            },
        }
    }

    /// Creates a subscription by periodically emitting a cloned message.
    pub fn interval(duration: Duration, message: Message) -> Self
    where
        Message: Clone,
    {
        Self::interval_with(duration, move || message.clone())
    }

    /// Creates a subscription by periodically invoking the provided closure.
    pub fn interval_with<F>(duration: Duration, mut message_factory: F) -> Self
    where
        F: FnMut() -> Message + Send + 'static,
    {
        let stream = stream! {
            loop {
                Delay::new(duration).await;
                yield message_factory();
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
        let token = self.token.clone();
        let mapped_stream = self.stream.map(f);
        Subscription::from_stream(mapped_stream).with_token_option(token)
    }

    /// Attaches a token so the runtime can detect identical subscriptions.
    pub fn with_token<T>(mut self, token: T) -> Self
    where
        T: PartialEq + Send + Sync + 'static,
    {
        self.token = Some(SubscriptionToken::new(token));
        self
    }

    fn with_token_option(mut self, token: Option<SubscriptionToken>) -> Self {
        self.token = token;
        self
    }

    fn identity(&self) -> Option<SubscriptionToken> {
        self.token.clone()
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

impl<Message> IntoSubscription<Message> for Subscription<Message>
where
    Message: Send + 'static,
{
    type Stream = Pin<Box<dyn Stream<Item = Message> + Send>>;

    fn identity(&self) -> Option<SubscriptionToken> {
        self.identity()
    }

    fn into_stream(self) -> Self::Stream {
        self.stream
    }
}

/// Subscription backed by a concrete stream type, avoiding boxing.
pub struct StreamSubscription<Message, S>
where
    Message: Send + 'static,
    S: Stream<Item = Message> + Send + 'static,
{
    stream: S,
    token: Option<SubscriptionToken>,
}

impl<Message, S> StreamSubscription<Message, S>
where
    Message: Send + 'static,
    S: Stream<Item = Message> + Send + 'static,
{
    /// Wraps the provided stream.
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            token: None,
        }
    }

    /// Attaches a token to this subscription.
    pub fn with_token<T>(mut self, token: T) -> Self
    where
        T: PartialEq + Send + Sync + 'static,
    {
        self.token = Some(SubscriptionToken::new(token));
        self
    }

    /// Converts the typed subscription into the boxed variant.
    pub fn boxed(self) -> Subscription<Message> {
        Subscription {
            stream: Box::pin(self.stream),
            token: self.token,
        }
    }
}

impl<Message, S> IntoSubscription<Message> for StreamSubscription<Message, S>
where
    Message: Send + 'static,
    S: Stream<Item = Message> + Send + 'static,
{
    type Stream = S;

    fn identity(&self) -> Option<SubscriptionToken> {
        self.token.clone()
    }

    fn into_stream(self) -> Self::Stream {
        self.stream
    }
}

/// Identifier used to compare subscriptions across renders.
#[derive(Clone)]
pub struct SubscriptionToken {
    inner: Arc<dyn TokenValue>,
}

impl SubscriptionToken {
    pub fn new<T>(value: T) -> Self
    where
        T: PartialEq + Send + Sync + 'static,
    {
        Self {
            inner: Arc::new(TokenValueImpl(value)),
        }
    }
}

impl PartialEq for SubscriptionToken {
    fn eq(&self, other: &Self) -> bool {
        self.inner.equals(other.inner.as_ref())
    }
}

impl Eq for SubscriptionToken {}

trait TokenValue: Send + Sync {
    fn equals(&self, other: &dyn TokenValue) -> bool;
    fn as_any(&self) -> &dyn Any;
}

struct TokenValueImpl<T>(T);

impl<T> TokenValue for TokenValueImpl<T>
where
    T: PartialEq + Send + Sync + 'static,
{
    fn equals(&self, other: &dyn TokenValue) -> bool {
        other
            .as_any()
            .downcast_ref::<TokenValueImpl<T>>()
            .map(|other| other.0 == self.0)
            .unwrap_or(false)
    }

    fn as_any(&self) -> &dyn Any {
        self
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
        let subscription = Subscription::interval(Duration::from_millis(5), 42);
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

    #[test]
    fn interval_with_produces_fresh_messages() {
        let mut counter = 0;
        let subscription = Subscription::interval_with(Duration::from_millis(5), move || {
            counter += 1;
            counter
        });
        let mut stream = subscription.into_stream();
        let values = block_on(async {
            let mut values = Vec::new();
            while let Some(value) = stream.next().await {
                values.push(value);
                if values.len() == 3 {
                    break;
                }
            }
            values
        });

        assert_eq!(values, vec![1, 2, 3]);
    }
}
