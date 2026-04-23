//! Golden-file parity: the same fixture scenes used by the glass goldens are
//! rendered through the text compositor with snapshot testing. Ensures
//! cross-surface coherence — any IR change that breaks parity lights up in
//! both surfaces.

use prosopon_compositor_text::{RenderOptions, render_scene};
use prosopon_core::Scene;

fn load(name: &str) -> Scene {
    let path = format!(
        "{}/../prosopon-compositor-glass/web/tests/fixtures/{}.json",
        env!("CARGO_MANIFEST_DIR"),
        name
    );
    let text = std::fs::read_to_string(&path).expect("fixture exists");
    serde_json::from_str(&text).expect("fixture parses")
}

#[test]
fn demo_scene_snapshot() {
    let scene = load("demo_scene");
    let out = render_scene(&scene, 80, &RenderOptions::plain());
    insta::assert_snapshot!(out);
}

#[test]
fn tool_flow_snapshot() {
    let scene = load("tool_flow");
    let out = render_scene(&scene, 80, &RenderOptions::plain());
    insta::assert_snapshot!(out);
}

#[test]
fn streaming_tokens_snapshot() {
    let scene = load("streaming_tokens");
    let out = render_scene(&scene, 80, &RenderOptions::plain());
    insta::assert_snapshot!(out);
}
