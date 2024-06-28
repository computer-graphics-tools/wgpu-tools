# WGPU Tools

<p align="left">
    <img src="media/WGPUTools.png", width="160">
</p>

A Rust library providing utility functions and abstractions for working with [WGPU](https://wgpu.rs), similar to [MetalTools](https://github.com/computer-graphics-tools/metal-tools).

## Features

- Context creation and management
- Texture handling utilities
- GPU operation scheduling
- Error handling

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
wgpu-tools = "0.0.1"
```

## Usage

Here are some examples of how to use wgpu-tools:

### Creating a standalone context

```rust
use wgpu_tools::Context;

async fn create_context() -> Result<Context, wgpu_tools::Error> {
    Context::default().await
}
```

### Creating a context with a surface

```rust
use wgpu_tools::Context;

async fn create_context_with_surface(window: &impl raw_window_handle::HasRawWindowHandle) -> Result<(Context, wgpu::Surface), wgpu_tools::Error> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
    let surface = unsafe { instance.create_surface(window) }?;
    let context = Context::default_with_surface(instance, Some(&surface)).await?;
    Ok((context, surface))
}
```

### Scheduling GPU operations

```rust
use wgpu_tools::Context;

fn schedule_gpu_operations(context: &Context) {
    context.schedule(|encoder| {
        // Perform GPU operations using the encoder
        // For example:
        let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[],
            depth_stencil_attachment: None,
        });
        // Add more GPU commands as needed
    });
}
```

### Loading a texture from an image file

```rust
use wgpu_tools::Context;
use image::io::Reader as ImageReader;

fn load_texture_from_file(context: &Context, path: &std::path::Path) -> Result<wgpu_tools::Texture, wgpu_tools::Error> {
    let img = ImageReader::open(path)?.decode()?;
    context.texture_from_image(&img, &wgpu::TextureFormat::Rgba8UnormSrgb, Some("Loaded Texture"))
}
```

## Main Components

### Context

The `Context` struct provides a convenient wrapper around wgpu's instance, adapter, device, and queue. It offers methods for creating contexts, scheduling GPU operations, and creating textures.

### Texture

The `Texture` struct encapsulates wgpu texture, view, and sampler objects.

### Error Handling

Custom error types are provided to handle various failure scenarios in wgpu operations.

## License

WGPUTools is licensed under [MIT license](LICENSE).