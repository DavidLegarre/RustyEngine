# RustyEngine — Rust + wgpu Game Engine Roadmap

Build RustyEngine as a small, testable game engine, then use it to build an original voxel sandbox inspired by Minecraft. Use `wgpu` with WebGPU as the rendering API. `wgpu` automatically selects the best backend per platform: Vulkan on Linux, Metal on macOS, DX12 on Windows, WebGL/WebGPU on the web. Revisit direct Vulkan through `ash` only if profiling or a concrete Vulkan-specific requirement justifies it.

## Goals and boundaries

### Engine goals

- [x] Provide window, input, timing, and application lifecycle support.
- [x] Render through `wgpu` with `Backends::all()` (Vulkan / Metal / DX12 / WebGL).
- [ ] Provide asset loading, logging, configuration, and diagnostics.
- [ ] Support voxel-world data, chunks, meshing, lighting, and persistence.
- [ ] Keep the engine small and testable. Add abstractions only when a real use case needs them.

### First game goals

- [ ] Build a single-player, first-person voxel world.
- [ ] Support procedural terrain, block placement and removal, inventory, and save/load.
- [ ] Use original names, textures, sounds, UI, and world content.
- [ ] Defer multiplayer, modding, and advanced simulation until the single-player loop works.

### Product boundary

Build an original voxel sandbox. Do not use Minecraft's code, assets, branding, or distinctive presentation. This is a product boundary, not legal advice.

## Architecture

```text
rustyengine/
  Cargo.toml
  crates/
    engine/             # reusable engine library
    game/               # voxel sandbox executable
  assets/
  docs/
```

```text
engine/
  app/                  # lifecycle and main loop
  platform/             # window and input
  render/               # wgpu device, resources, and passes
  asset/                # asset loading
  world/                # voxel primitives and chunk data
  diagnostics/          # logs, metrics, and debug UI
```

Keep game-specific rules in `game`: terrain generation, block types, player controls, inventory, crafting, and game saves. Engine modules should not know what a dirt block or crafting recipe means.

Prefer clear APIs and data flow over an entity-component system at the start. Add an ECS only when gameplay needs one. Do not design plugin, networking, or mod interfaces before a real use case appears.

## WebGPU backend decision

`wgpu::Backends::all()` enables Vulkan, Metal, DX12, and WebGL/WebGPU. On Linux, Vulkan is selected. On macOS, Metal. On Windows, DX12. On the web, WebGL.

- **Primary target:** Vulkan (Linux desktop, Linux gaming).
- **Fallback:** Metal (macOS), DX12 (Windows), WebGL (browser).
- **Do not** pin to `Backends::VULKAN`. Pinning blocks cross-platform support.
- **Do not** write a raw Vulkan renderer unless a concrete need appears.

Set the backend with:

```rust
let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
    backends: wgpu::Backends::all(),
    ..wgpu::InstanceDescriptor::new_without_display_handle()
});
```

Force a specific backend at runtime with the `WGPU_BACKEND` environment variable: `vulkan`, `metal`, `dx12`, `gl`, `webgpu`.

## Phased implementation

### Phase 0 — Make the project build

- [x] Check the current tree and Cargo configuration.
- [x] Resolve the missing `app` module: add a minimal module or remove the declaration.
- [x] Add a basic executable that exits cleanly.
- [x] `cargo check --workspace` passes.
- [ ] `cargo fmt` runs cleanly on all sources.
- [ ] `cargo test` runs (even if empty).
- [ ] Add build and run commands to the README.

**Done when:** a clean checkout builds, formats, and tests with documented commands.

### Phase 1 — Set the foundation and make key decisions

- [x] Set up a Cargo workspace with an `engine` library.
- [x] Keep dependencies minimal and commit `Cargo.lock`.
- [x] Select core libraries:
  - [x] `winit` for window and event handling.
  - [x] `wgpu` for rendering, configured with `Backends::all()`.
  - [ ] `glam` for vectors and matrices.
  - [ ] `tracing` + `tracing-subscriber` for structured logs.
  - [ ] `bytemuck` for GPU data layout where needed.
  - [ ] `image` for common image formats.
- [ ] Choose supported platforms and minimum GPU requirements.
- [ ] Return errors from initialization and asset loading. Log recoverable runtime failures.
- [ ] Add CI checks for formatting, linting, tests, and builds.

**Done when:** the workspace builds on target platforms, WebGPU backend selection is explicit, and dependency choices are recorded.

### Phase 2 — Window, lifecycle, and diagnostics

- [x] Create a window and handle close and resize events.
- [x] Define application lifecycle: initialize, update, render, and shutdown.
- [ ] Use a fixed or capped timestep for simulation and a variable timestep for rendering.
- [ ] Track frame time, update time, and frames per second.
- [ ] Handle keyboard, mouse, focus, and cursor capture.
- [ ] Handle window minimization and surface reconfiguration.
- [ ] Log the GPU adapter, backend, and renderer limits at startup.

**Done when:** the app opens, resizes, closes cleanly, and reports useful startup diagnostics.

### Phase 3 — WebGPU rendering foundation

- [x] Create the `wgpu` instance, surface, adapter, device, queue, and surface configuration.
- [x] Handle surface loss, outdated surfaces, and zero-size windows.
- [x] Clear the screen with a known color.
- [ ] Add a shader pipeline and render a colored triangle.
- [ ] Add camera matrices and draw a textured cube.
- [ ] Load shaders and textures from project assets.
- [ ] Enable validation and useful GPU error reporting in development builds.
- [ ] Keep GPU resource creation and lifetime inside the renderer. Expose higher-level render operations to the game.

**Done when:** a camera can move around a textured cube, and resize or minimize does not crash the app.

### Phase 4 — Core rendering and asset APIs

- [ ] Define small renderer-facing types for meshes, textures, materials, and camera data.
- [ ] Add vertex and index buffers, depth testing, back-face culling, and texture sampling.
- [ ] Implement a basic render-pass structure for opaque world geometry.
- [ ] Add asset paths and loading errors. Add development-time reload only if it stays simple.
- [ ] Reuse GPU resources; do not create them every frame.
- [ ] Add a debug mode for adapter information, frame timings, and draw statistics.

**Done when:** game code can submit reusable mesh and material data without reaching into Vulkan setup.

### Phase 5 — Voxel data model

- [ ] Define block IDs and properties: solidity, transparency, render layer, texture, and collision behavior.
- [ ] Start with chunks of **16×16×16** blocks.
- [ ] Store blocks in compact arrays with consistent world-to-chunk coordinate conversion.
- [ ] Define chunk states: empty, generated, dirty, meshed, and ready for upload.
- [ ] Keep world data separate from GPU mesh data.
- [ ] Write tests for coordinate conversion, negative coordinates, chunk boundaries, and block indexing.

**Done when:** world data can be queried and changed correctly across chunk boundaries.

### Phase 6 — Chunk meshing and world rendering

- [ ] Implement a basic face-culling mesher. Emit faces only when adjacent blocks do not occlude them.
- [ ] Generate normals, UVs, texture layer, and block-light attributes per vertex.
- [ ] Use a texture atlas or texture array. Prefer texture arrays if target hardware supports them.
- [ ] Upload chunk meshes to the GPU and draw visible chunks.
- [ ] Rebuild only dirty chunks after block edits.
- [ ] Add frustum culling.
- [ ] Measure mesh size, generation time, draw calls, and frame time.

Start with one mesh per chunk and opaque blocks only. Add greedy meshing only after profiling shows basic face culling is too expensive.

**Done when:** a test world renders as chunks, and changing a block updates only affected chunk meshes.

### Phase 7 — Player movement and interaction

- [ ] Add a first-person camera with mouse look.
- [ ] Implement player movement, gravity, jumping, and collision against solid blocks.
- [ ] Use a simple player capsule or AABB collision model.
- [ ] Add block selection with voxel ray traversal, such as Amanatides–Woo.
- [ ] Support breaking and placing blocks. Check reach, collision, and valid target cells.
- [ ] Write tests for ray hits, movement collisions, and placement edge cases.

**Done when:** the player can walk, jump, target blocks, remove blocks, and place blocks without entering solid terrain.

### Phase 8 — Terrain generation and streaming

- [ ] Add deterministic, seeded terrain generation.
- [ ] Start with height-based terrain and a small set of block types.
- [ ] Generate chunks around the player and unload distant chunks.
- [ ] Move generation and meshing work off the main thread.
- [ ] Send completed CPU data back to the main thread for GPU upload.
- [ ] Limit per-frame uploads to avoid frame spikes.
- [ ] Add cancellation and stale-result handling when the player moves quickly.
- [ ] Track generation queue depth and chunk load times.

**Done when:** the player can move through a larger generated world without blocking the main thread or causing unbounded memory growth.

### Phase 9 — Lighting and visual quality

- [ ] Add basic directional or ambient lighting.
- [ ] Add block light and sunlight after world generation and meshing are stable.
- [ ] Store lighting data in chunk cells and update neighboring chunks when light changes.
- [ ] Add fog to hide the world-streaming horizon.
- [ ] Add sky color and a simple day/night cycle only if desired for the first release.
- [ ] Profile lighting updates and their effect on mesh rebuilds.

Keep lighting scope controlled. Full Minecraft-style light propagation is a separate subsystem with tricky chunk-boundary behavior.

**Done when:** the world has readable depth and lighting updates remain correct at chunk edges.

### Phase 10 — Inventory, UI, and gameplay loop

- [ ] Add block selection and a hotbar.
- [ ] Keep the inventory model independent of UI rendering.
- [ ] Add original icons, fonts, and interface layout.
- [ ] Add a pause menu, settings, and input rebinding only as needed.
- [ ] Add basic crafting only if it belongs in the intended first game.
- [ ] Keep game state separate from renderer state.

**Done when:** the player can select blocks and complete the core build-and-explore loop.

### Phase 11 — Save/load and world stability

- [ ] Define a versioned save format for world seed, modified chunks, player position, and inventory.
- [ ] Store only modified data if unchanged chunks can be regenerated reliably.
- [ ] Write saves atomically: write a temporary file, flush it, then rename it.
- [ ] Validate save data before loading. Handle missing or corrupt files safely.
- [ ] Define a migration strategy before changing the format after release.
- [ ] Test interrupted saves, invalid data, and different world versions.

**Done when:** a world can close and reopen with player changes preserved, and corruption does not destroy existing saves.

### Phase 12 — Performance and release readiness

- [ ] Profile on representative low-, mid-, and high-end hardware.
- [ ] Set practical budgets for frame time, visible chunks, memory, and chunk generation.
- [ ] Optimize from measurements. Consider greedy meshing, draw batching, compression, or LOD only when needed.
- [ ] Test long sessions, rapid movement, repeated edits, and world reloads.
- [ ] Add crash logs and graphics-device failure messages.
- [ ] Document controls, system requirements, build steps, and known limitations.
- [ ] Package original assets with clear licenses.

**Done when:** performance stays within chosen budgets on target hardware and release builds pass smoke tests.

## First playable milestone

Keep the first playable deliberately small:

- [ ] WebGPU-backed window and triangle.
- [ ] Textured cube and first-person camera.
- [ ] One chunk of test blocks.
- [ ] Face-culling mesh generation.
- [ ] Walk, look, place, and remove blocks.
- [ ] Deterministic flat or height-based terrain.
- [ ] Save/load one world.

Do not start with crafting, mobs, multiplayer, modding, or a complex ECS. Those features do not prove the engine's core path.

## Main risks and controls

- [ ] **WebGPU scope grows quickly:** start with `wgpu`'s multi-backend mode. Do not write a raw Vulkan renderer unless a concrete need appears.
- [ ] **Chunk work causes frame spikes:** separate CPU generation from GPU upload, and budget uploads per frame.
- [ ] **World-coordinate bugs:** test negative coordinates and chunk boundaries before adding streaming.
- [ ] **Lighting becomes a major project:** defer advanced propagation and keep initial lighting simple.
- [ ] **Save bugs destroy progress:** version saves and use atomic writes.
- [ ] **Large architecture slows gameplay:** keep engine APIs small and add abstractions only when real use cases need them.

## Deferred scope

The first playable does not include a raw `ash` renderer, multiplayer, modding, mobs, advanced crafting, or large-world LOD. Reconsider these only after the core playable loop works and a concrete requirement exists.
