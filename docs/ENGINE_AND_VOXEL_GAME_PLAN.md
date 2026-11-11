# RustyEngine and Voxel Sandbox Roadmap

## Purpose

Build RustyEngine as a small, testable game engine, then use it to build an original voxel sandbox inspired by Minecraft. Use `wgpu` with its Vulkan backend first. This provides a modern explicit GPU API while avoiding the cost of implementing every low-level Vulkan detail immediately. Revisit direct Vulkan through `ash` only if profiling or a concrete Vulkan-specific requirement justifies it.

The repository is currently a stub. `Cargo.toml` names the Rust 2024 project, `README.md` describes it as Vulkan-based, and `src/main.rs` declares a missing `app` module. Make the project build before expanding the engine.

## Goals and boundaries

### Engine goals

- Provide window, input, timing, and application lifecycle support.
- Render through `wgpu` using Vulkan as the required backend.
- Provide asset loading, logging, configuration, and diagnostics.
- Support voxel-world data, chunks, meshing, lighting, and persistence.
- Keep the engine small and testable. Add abstractions only when a real use case needs them.

### First game goals

- Build a single-player, first-person voxel world.
- Support procedural terrain, block placement and removal, inventory, and save/load.
- Use original names, textures, sounds, UI, and world content.
- Defer multiplayer, modding, and advanced simulation until the single-player loop works.

### Product boundary

Build an original voxel sandbox. Do not use Minecraft's code, assets, branding, or distinctive presentation. This is a product boundary, not legal advice.

## Architecture

Keep engine and game code separate:

```text
rustyengine/
  Cargo.toml
  crates/
    engine/             # reusable engine library
    game/               # voxel sandbox executable
  assets/
  docs/
```

Split engine code by responsibility as those responsibilities appear:

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

## Phased implementation

### Phase 0 — Make the project build

1. Check the current tree and Cargo configuration.
2. Resolve the missing `app` module: add a minimal module or remove the declaration.
3. Add a basic executable that exits cleanly.
4. Run `cargo fmt`, `cargo check`, and `cargo test`.
5. Add build and run commands to the README.

**Done when:** a clean checkout builds and tests with documented commands.

### Phase 1 — Set the foundation and make key decisions

1. Set up a Cargo workspace with an `engine` library and a `game` executable.
2. Keep dependencies minimal and commit `Cargo.lock`.
3. Select core libraries:
   - `winit` for window and event handling.
   - `wgpu` for rendering, configured to use Vulkan as the required backend.
   - `glam` for vectors and matrices.
   - `tracing` for structured logs.
   - `bytemuck` for GPU data layout where needed.
   - `image` for common image formats.
4. Choose supported platforms and minimum Vulkan/GPU requirements.
5. Return errors from initialization and asset loading. Log recoverable runtime failures.
6. Add CI checks for formatting, linting, tests, and builds.

**Vulkan decision:** `wgpu` provides a modern explicit GPU API and uses Vulkan without requiring the project to own a raw Vulkan renderer. If direct Vulkan control is a hard requirement, choose `ash` before implementing the renderer. Do not support both rendering paths initially.

**Done when:** the workspace builds on target platforms, Vulkan backend selection is explicit, and dependency choices are recorded.

### Phase 2 — Window, lifecycle, and diagnostics

1. Create a window and handle close and resize events.
2. Define application lifecycle: initialize, update, render, and shutdown.
3. Use a fixed or capped timestep for simulation and a variable timestep for rendering.
4. Track frame time, update time, and frames per second.
5. Handle keyboard, mouse, focus, and cursor capture.
6. Handle window minimization and surface reconfiguration.
7. Log the GPU adapter, backend, and renderer limits at startup.

**Done when:** the app opens, resizes, closes cleanly, and reports useful startup diagnostics.

### Phase 3 — Vulkan-backed rendering foundation

1. Create the `wgpu` instance, Vulkan surface, adapter, device, queue, and surface configuration.
2. Handle surface loss, outdated surfaces, and zero-size windows.
3. Clear the screen with a known color.
4. Add a shader pipeline and render a colored triangle.
5. Add camera matrices and draw a textured cube.
6. Load shaders and textures from project assets.
7. Enable validation and useful GPU error reporting in development builds.
8. Keep GPU resource creation and lifetime inside the renderer. Expose higher-level render operations to the game.

**Done when:** a camera can move around a textured cube, and resize or minimize does not crash the app.

### Phase 4 — Core rendering and asset APIs

1. Define small renderer-facing types for meshes, textures, materials, and camera data.
2. Add vertex and index buffers, depth testing, back-face culling, and texture sampling.
3. Implement a basic render-pass structure for opaque world geometry.
4. Add asset paths and loading errors. Add development-time reload only if it stays simple.
5. Reuse GPU resources; do not create them every frame.
6. Add a debug mode for adapter information, frame timings, and draw statistics.

**Done when:** game code can submit reusable mesh and material data without reaching into Vulkan setup.

### Phase 5 — Voxel data model

1. Define block IDs and properties: solidity, transparency, render layer, texture, and collision behavior.
2. Start with chunks of **16×16×16** blocks.
3. Store blocks in compact arrays with consistent world-to-chunk coordinate conversion.
4. Define chunk states: empty, generated, dirty, meshed, and ready for upload.
5. Keep world data separate from GPU mesh data.
6. Test coordinate conversion, negative coordinates, chunk boundaries, and block indexing.

**Done when:** world data can be queried and changed correctly across chunk boundaries.

### Phase 6 — Chunk meshing and world rendering

1. Implement a basic face-culling mesher. Emit faces only when adjacent blocks do not occlude them.
2. Generate normals, UVs, texture layer, and block-light attributes per vertex.
3. Use a texture atlas or texture array. Prefer texture arrays if target hardware supports them.
4. Upload chunk meshes to the GPU and draw visible chunks.
5. Rebuild only dirty chunks after block edits.
6. Add frustum culling.
7. Measure mesh size, generation time, draw calls, and frame time.

Start with one mesh per chunk and opaque blocks only. Add greedy meshing only after profiling shows basic face culling is too expensive.

**Done when:** a test world renders as chunks, and changing a block updates only affected chunk meshes.

### Phase 7 — Player movement and interaction

1. Add a first-person camera with mouse look.
2. Implement player movement, gravity, jumping, and collision against solid blocks.
3. Use a simple player capsule or AABB collision model.
4. Add block selection with voxel ray traversal, such as Amanatides–Woo.
5. Support breaking and placing blocks. Check reach, collision, and valid target cells.
6. Test ray hits, movement collisions, and placement edge cases.

**Done when:** the player can walk, jump, target blocks, remove blocks, and place blocks without entering solid terrain.

### Phase 8 — Terrain generation and streaming

1. Add deterministic, seeded terrain generation.
2. Start with height-based terrain and a small set of block types.
3. Generate chunks around the player and unload distant chunks.
4. Move generation and meshing work off the main thread.
5. Send completed CPU data back to the main thread for GPU upload.
6. Limit per-frame uploads to avoid frame spikes.
7. Add cancellation and stale-result handling when the player moves quickly.
8. Track generation queue depth and chunk load times.

**Done when:** the player can move through a larger generated world without blocking the main thread or causing unbounded memory growth.

### Phase 9 — Lighting and visual quality

1. Add basic directional or ambient lighting.
2. Add block light and sunlight after world generation and meshing are stable.
3. Store lighting data in chunk cells and update neighboring chunks when light changes.
4. Add fog to hide the world-streaming horizon.
5. Add sky color and a simple day/night cycle only if desired for the first release.
6. Profile lighting updates and their effect on mesh rebuilds.

Keep lighting scope controlled. Full Minecraft-style light propagation is a separate subsystem with tricky chunk-boundary behavior.

**Done when:** the world has readable depth and lighting updates remain correct at chunk edges.

### Phase 10 — Inventory, UI, and gameplay loop

1. Add block selection and a hotbar.
2. Keep the inventory model independent of UI rendering.
3. Add original icons, fonts, and interface layout.
4. Add a pause menu, settings, and input rebinding only as needed.
5. Add basic crafting only if it belongs in the intended first game.
6. Keep game state separate from renderer state.

**Done when:** the player can select blocks and complete the core build-and-explore loop.

### Phase 11 — Save/load and world stability

1. Define a versioned save format for world seed, modified chunks, player position, and inventory.
2. Store only modified data if unchanged chunks can be regenerated reliably.
3. Write saves atomically: write a temporary file, flush it, then rename it.
4. Validate save data before loading. Handle missing or corrupt files safely.
5. Define a migration strategy before changing the format after release.
6. Test interrupted saves, invalid data, and different world versions.

**Done when:** a world can close and reopen with player changes preserved, and corruption does not destroy existing saves.

### Phase 12 — Performance and release readiness

1. Profile on representative low-, mid-, and high-end hardware.
2. Set practical budgets for frame time, visible chunks, memory, and chunk generation.
3. Optimize from measurements. Consider greedy meshing, draw batching, compression, or LOD only when needed.
4. Test long sessions, rapid movement, repeated edits, and world reloads.
5. Add crash logs and graphics-device failure messages.
6. Document controls, system requirements, build steps, and known limitations.
7. Package original assets with clear licenses.

**Done when:** performance stays within chosen budgets on target hardware and release builds pass smoke tests.

## First playable milestone

Keep the first playable deliberately small:

- Vulkan-backed window and triangle.
- Textured cube and first-person camera.
- One chunk of test blocks.
- Face-culling mesh generation.
- Walk, look, place, and remove blocks.
- Deterministic flat or height-based terrain.
- Save/load one world.

Do not start with crafting, mobs, multiplayer, modding, or a complex ECS. Those features do not prove the engine's core path.

## Main risks and controls

- **Vulkan scope grows quickly:** start with `wgpu`'s Vulkan backend. Do not write a raw Vulkan renderer unless a concrete need appears.
- **Chunk work causes frame spikes:** separate CPU generation from GPU upload, and budget uploads per frame.
- **World-coordinate bugs:** test negative coordinates and chunk boundaries before adding streaming.
- **Lighting becomes a major project:** defer advanced propagation and keep initial lighting simple.
- **Save bugs destroy progress:** version saves and use atomic writes.
- **Large architecture slows gameplay:** keep engine APIs small and add abstractions only when real use cases need them.

## Deferred scope

The first playable does not include a raw `ash` renderer, multiplayer, modding, mobs, advanced crafting, or large-world LOD. Reconsider these only after the core playable loop works and a concrete requirement exists.
