//! Integration test: start a GlassServer on 127.0.0.1:0, connect via WS, send
//! a SceneReset through the compositor, and assert it arrives on the wire.

use futures::StreamExt;
use prosopon_compositor_glass::{GlassCompositor, GlassServer, GlassServerConfig};
use prosopon_core::{Intent, Node, ProsoponEvent, Scene};
use prosopon_runtime::Compositor;
use tokio_tungstenite::connect_async;

#[tokio::test]
async fn ws_client_receives_envelopes() {
    let server = GlassServer::bind(GlassServerConfig {
        addr: "127.0.0.1:0".parse().unwrap(),
    })
    .await
    .expect("bind succeeds");
    let url = format!("ws://{}/ws", server.local_addr());
    let mut compositor = GlassCompositor::new(server.fanout());
    let serve = tokio::spawn(server.serve());

    // Allow the server to start accepting.
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let (mut ws, _resp) = connect_async(&url).await.expect("ws connect");

    // Give the WS upgrade a beat to subscribe before we send.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let scene = Scene::new(Node::new(Intent::Prose {
        text: "hello".into(),
    }));
    compositor
        .apply(&ProsoponEvent::SceneReset { scene })
        .unwrap();

    let msg = tokio::time::timeout(std::time::Duration::from_secs(2), ws.next())
        .await
        .expect("got a message in time")
        .expect("stream not closed")
        .expect("ws frame");
    let text = msg.into_text().unwrap().to_string();
    assert!(text.contains("\"scene_reset\""), "frame was: {text}");

    serve.abort();
}
