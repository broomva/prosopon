//! The compositor MUST accept every ProsoponEvent variant without erroring or
//! panicking. It MAY no-op for events it surfaces downstream (no browser connected).

use prosopon_compositor_glass::GlassCompositor;
use prosopon_core::{
    ChunkPayload, Intent, Node, NodePatch, ProsoponEvent, Scene, SignalValue, StreamChunk, Topic,
};
use prosopon_runtime::Compositor;

fn scene() -> Scene {
    Scene::new(Node::new(Intent::Prose { text: "hi".into() }).with_id("root"))
}

fn all_events() -> Vec<ProsoponEvent> {
    vec![
        ProsoponEvent::SceneReset { scene: scene() },
        ProsoponEvent::NodeAdded {
            parent: prosopon_core::NodeId::from_raw("root"),
            node: Node::new(Intent::Prose { text: "x".into() }),
        },
        ProsoponEvent::NodeUpdated {
            id: prosopon_core::NodeId::from_raw("root"),
            patch: NodePatch::default(),
        },
        ProsoponEvent::NodeRemoved {
            id: prosopon_core::NodeId::from_raw("root"),
        },
        ProsoponEvent::SignalChanged {
            topic: Topic::new("t"),
            value: SignalValue::Scalar(serde_json::json!(1.0)),
            ts: chrono::Utc::now(),
        },
        ProsoponEvent::StreamChunk {
            id: prosopon_core::StreamId::from_raw("s"),
            chunk: StreamChunk {
                seq: 1,
                payload: ChunkPayload::Text { text: "tok".into() },
                final_: true,
            },
        },
        ProsoponEvent::Heartbeat {
            ts: chrono::Utc::now(),
        },
    ]
}

#[tokio::test]
async fn apply_is_total_over_events() {
    let mut c = GlassCompositor::detached();
    for e in all_events() {
        c.apply(&e).expect("every event must be accepted");
    }
    c.flush().expect("flush never errors");
}

#[tokio::test]
async fn capabilities_advertise_twod() {
    let c = GlassCompositor::detached();
    let caps = c.capabilities();
    assert!(caps.surfaces.contains(&prosopon_core::SurfaceKind::TwoD));
    assert!(caps.supports_streaming);
    assert!(caps.supports_signal_push);
}
