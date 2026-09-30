use vello::wgpu::{
    Device, Extent3d, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    TextureView,
};

pub const CANVAS: TextureUsages = TextureUsages::RENDER_ATTACHMENT
    .union(TextureUsages::TEXTURE_BINDING)
    .union(TextureUsages::COPY_SRC);

pub const SCRATCH: TextureUsages =
    TextureUsages::STORAGE_BINDING.union(TextureUsages::TEXTURE_BINDING);

// what the commands draw onto, one premultiplied color per real pixel
pub fn canvas(device: &Device, width: u32, height: u32) -> Texture {
    create(device, width, height, CANVAS)
}

// vello writes here from a compute shader, with plain, not premultiplied, alpha
pub fn scratch(device: &Device, width: u32, height: u32) -> Texture {
    create(device, width, height, SCRATCH)
}

// vello copies the finished blur into its image atlas, so it has to be a copy source
pub fn blur(device: &Device, width: u32, height: u32) -> Texture {
    let usage = TextureUsages::RENDER_ATTACHMENT
        | TextureUsages::TEXTURE_BINDING
        | TextureUsages::COPY_SRC
        | TextureUsages::COPY_DST;

    create(device, width, height, usage)
}

pub fn view(texture: &Texture) -> TextureView {
    texture.create_view(&Default::default())
}

// new textures start out fully transparent
fn create(device: &Device, width: u32, height: u32, usage: TextureUsages) -> Texture {
    let size = Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    device.create_texture(&TextureDescriptor {
        label: None,
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage,
        view_formats: &[],
    })
}
