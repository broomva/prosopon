# prosopon-compositor-glass

2D web compositor for the Prosopon display server. Renders `prosopon_core::Intent`
into an Arcan Glass-styled Preact UI, served from an embedded bundle or
consumable as a TypeScript package.

## Run the dev server

    cargo run -p prosopon-compositor-glass --bin prosopon-glass -- serve \
        --port 4321 --fixture tests/fixtures/demo_scene.json

Open http://localhost:4321/.

## Use as a Compositor

    use prosopon_compositor_glass::{GlassCompositor, GlassServer};
    use prosopon_runtime::Runtime;

    let mut server = GlassServer::bind("127.0.0.1:4321").await?;
    let compositor = GlassCompositor::new(server.fanout());
    runtime.register_compositor(Box::new(compositor)).await;
    server.serve().await?;

## Architecture

See `docs/surfaces/glass.md` and `docs/rfcs/0002-compositor-contract.md`.
