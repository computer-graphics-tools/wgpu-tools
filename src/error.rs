#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("requesting adapter failed")]
    RequestingAdapterFailed,
    #[error(transparent)]
    RequestingDeviceFailed(#[from] wgpu::RequestDeviceError),
    #[error(transparent)]
    ImageError(#[from] image::ImageError),
    #[error("texture creation failed")]
    TextureCreationFailed,
}
