use std::error::Error;

#[allow(dead_code)]
pub trait HotModuleReload {
    fn start() -> Result<(), Box<dyn Error>>;
    fn stop();
}
