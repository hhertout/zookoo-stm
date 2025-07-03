use crate::Scraping;

pub struct Scrapper {}

impl Scraping for Scrapper {
    fn send_request(&self) -> Result<(), ()> {
        Ok(())
    }
}
