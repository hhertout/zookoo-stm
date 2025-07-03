use std::{collections::HashMap, io::Error};

pub mod config;
pub mod otel;

#[derive(Debug)]
pub struct ExporterRequest {
    pub exporter: ExporterConfigurationRequest,
    pub metrics: HashMap<String, isize>,
}

#[derive(Debug)]
pub struct ExporterConfigurationRequest {}

pub trait Export {
    fn export(&self, data: ExporterRequest) -> Result<(), Error>;
}
