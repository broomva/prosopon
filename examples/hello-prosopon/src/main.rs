//! Smallest end-to-end Prosopon example.
//!
//! An "agent" emits a scene with a progress signal, fires off a few signal updates,
//! and renders the whole thing via the text compositor.

use std::time::Duration;

use anyhow::Result;
use prosopon_compositor_text::{TextCompositor, TextTarget};
use prosopon_core::{
    Intent, Node, ProsoponEvent, Scene, SignalRef, SignalValue, Topic, json, signal::BindTarget,
    signal::Binding, signal::Transform,
};
use prosopon_runtime::Runtime;
use prosopon_sdk::{Session, ir};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .compact()
        .init();

    // 1. Build the initial scene.
    let scene = build_scene();

    // 2. Spin up a runtime with a text compositor attached.
    let runtime = Runtime::new(scene.clone());
    runtime
        .register_compositor(Box::new(TextCompositor::new(TextTarget::stdout(), 100)))
        .await;

    // 3. Fire the initial scene-reset event so the compositor renders.
    let mut session = Session::new();
    let env = session.envelope(ProsoponEvent::SceneReset {
        scene: scene.clone(),
    });
    tracing::info!(session = %env.session_id, seq = env.seq, "emitting scene reset");
    runtime.submit(env.event.clone()).await?;

    // 4. Pretend to do some work — publish signal updates over time.
    for i in 1..=10 {
        tokio::time::sleep(Duration::from_millis(120)).await;
        let pct = i as f32 / 10.0;
        runtime
            .publish_signal(
                Topic::new("hello.progress"),
                SignalValue::Scalar(json!(pct)),
            )
            .await;
        runtime
            .submit(ProsoponEvent::SignalChanged {
                topic: Topic::new("hello.progress"),
                value: SignalValue::Scalar(json!(pct)),
                ts: chrono::Utc::now(),
            })
            .await?;
    }

    println!();
    println!("Done. (Run `prosopon demo` for a denser example.)");
    Ok(())
}

fn build_scene() -> Scene {
    ir::section("Hello, Prosopon")
        .child(ir::prose(
            "Minimal Prosopon demo: an agent emitting one IR, rendered by the reference \
             text compositor. The progress bar below is bound to a live signal.",
        ))
        .child(ir::divider())
        .child(
            Node::new(Intent::Progress {
                pct: Some(0.0),
                label: Some("working".into()),
            })
            .bind(Binding {
                source: SignalRef::topic(Topic::new("hello.progress")),
                target: BindTarget::Attr { key: "pct".into() },
                transform: Some(Transform::Identity),
            }),
        )
        .child(ir::divider())
        .child(ir::prose(
            "Next steps: see the glass compositor for a web surface, and the field \
             compositor for Plexus-style shader visualization.",
        ))
        .into_scene()
}
