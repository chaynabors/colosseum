#[derive(Debug)]
pub enum Error {
    WindowCreationFailed,
    NoSuitableGraphicsAdapter,
    NoSuitableGraphicsDevice,
    IncompatibleSurface,
    SurfaceLost,
    OutOfMemory,
}
