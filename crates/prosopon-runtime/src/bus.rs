//! Signal bus — topic-keyed pub/sub with last-value caching.
//!
//! Compositors subscribe by topic prefix or exact match; agents publish values. The
//! bus caches the most recent value per topic so late subscribers can recover state.

use indexmap::IndexMap;
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};

use prosopon_core::{SignalValue, Topic};

const CHANNEL_CAPACITY: usize = 256;

/// A topic-addressed signal bus.
pub struct SignalBus {
    state: Mutex<BusState>,
}

struct BusState {
    last_known: IndexMap<Topic, SignalValue>,
    channels: IndexMap<Topic, broadcast::Sender<(Topic, SignalValue)>>,
    wildcard: broadcast::Sender<(Topic, SignalValue)>,
}

impl SignalBus {
    /// Create an empty bus.
    #[must_use]
    pub fn new() -> Self {
        let (wildcard, _) = broadcast::channel(CHANNEL_CAPACITY);
        Self {
            state: Mutex::new(BusState {
                last_known: IndexMap::new(),
                channels: IndexMap::new(),
                wildcard,
            }),
        }
    }

    /// Publish a new value on `topic`. Subscribers on exact match and wildcard
    /// subscribers both receive it. Last-value cache is updated.
    pub async fn publish(&self, topic: Topic, value: SignalValue) {
        let mut state = self.state.lock().await;
        state.last_known.insert(topic.clone(), value.clone());
        if let Some(sender) = state.channels.get(&topic) {
            let _ = sender.send((topic.clone(), value.clone()));
        }
        let _ = state.wildcard.send((topic, value));
    }

    /// Subscribe to a specific topic. Receives every future publish.
    pub async fn subscribe(&self, topic: Topic) -> SignalSubscriber {
        let mut state = self.state.lock().await;
        let sender = state
            .channels
            .entry(topic.clone())
            .or_insert_with(|| broadcast::channel(CHANNEL_CAPACITY).0);
        let rx = sender.subscribe();
        SignalSubscriber {
            topic: Some(topic),
            rx,
        }
    }

    /// Subscribe to every publish (wildcard). Use sparingly; each receiver gets
    /// a copy of every value.
    pub async fn subscribe_all(&self) -> SignalSubscriber {
        let state = self.state.lock().await;
        let rx = state.wildcard.subscribe();
        SignalSubscriber { topic: None, rx }
    }

    /// Read the last-known value of a topic.
    pub async fn last_known(&self, topic: &Topic) -> Option<SignalValue> {
        self.state.lock().await.last_known.get(topic).cloned()
    }

    /// Snapshot all last-known values.
    pub async fn snapshot(&self) -> IndexMap<Topic, SignalValue> {
        self.state.lock().await.last_known.clone()
    }
}

impl Default for SignalBus {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for SignalBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignalBus").finish_non_exhaustive()
    }
}

/// A subscription handle yielding future publishes.
pub struct SignalSubscriber {
    topic: Option<Topic>,
    rx: broadcast::Receiver<(Topic, SignalValue)>,
}

impl SignalSubscriber {
    /// The topic filter, or `None` for wildcard subscribers.
    #[must_use]
    pub fn topic(&self) -> Option<&Topic> {
        self.topic.as_ref()
    }

    /// Await the next message. Errors are rare (broadcast `Closed`/`Lagged`); callers
    /// who want lag handling should use [`SignalSubscriber::recv_raw`].
    ///
    /// # Errors
    /// Returns `BusRecvError` if the sender is dropped or the receiver lagged.
    pub async fn recv(&mut self) -> Result<(Topic, SignalValue), BusRecvError> {
        self.rx.recv().await.map_err(|e| match e {
            broadcast::error::RecvError::Closed => BusRecvError::Closed,
            broadcast::error::RecvError::Lagged(n) => BusRecvError::Lagged(n),
        })
    }
}

/// Errors returned by [`SignalSubscriber::recv`].
#[derive(Debug, thiserror::Error)]
pub enum BusRecvError {
    #[error("bus closed")]
    Closed,
    #[error("subscriber lagged by {0} messages")]
    Lagged(u64),
}

/// Public helper for typed `Arc<SignalBus>` sharing.
pub type SharedBus = Arc<SignalBus>;

#[cfg(test)]
mod tests {
    use super::*;
    use prosopon_core::json;

    #[tokio::test]
    async fn publish_updates_last_known() {
        let bus = SignalBus::new();
        let topic = Topic::new("plexus.load");
        bus.publish(topic.clone(), SignalValue::Scalar(json!(0.5)))
            .await;
        let v = bus.last_known(&topic).await.unwrap();
        assert!(matches!(v, SignalValue::Scalar(_)));
    }

    #[tokio::test]
    async fn specific_subscriber_receives_only_its_topic() {
        let bus = SignalBus::new();
        let t1 = Topic::new("a");
        let t2 = Topic::new("b");
        let mut sub = bus.subscribe(t1.clone()).await;
        bus.publish(t2.clone(), SignalValue::Scalar(json!(1))).await;
        bus.publish(t1.clone(), SignalValue::Scalar(json!(2))).await;
        let (recv_topic, _) = sub.recv().await.unwrap();
        assert_eq!(recv_topic, t1);
    }

    #[tokio::test]
    async fn wildcard_subscriber_receives_all() {
        let bus = SignalBus::new();
        let mut sub = bus.subscribe_all().await;
        bus.publish(Topic::new("x"), SignalValue::Scalar(json!(1)))
            .await;
        bus.publish(Topic::new("y"), SignalValue::Scalar(json!(2)))
            .await;
        let a = sub.recv().await.unwrap().0;
        let b = sub.recv().await.unwrap().0;
        assert_eq!(a, Topic::new("x"));
        assert_eq!(b, Topic::new("y"));
    }
}
