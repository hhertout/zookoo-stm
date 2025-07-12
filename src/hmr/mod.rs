use std::error::Error;

pub trait HotModuleReload {
    fn start() -> Result<(), Box<dyn Error>>;
    fn stop();
}
