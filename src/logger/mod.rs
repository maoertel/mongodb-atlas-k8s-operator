mod encoder;
pub(crate) mod error;

use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::Appender;
use log4rs::config::Root;
use log4rs::Config;
use log4rs::Handle;

use crate::logger::encoder::JsonEncoder;
use crate::logger::error::Result;

pub fn init() -> Result<Handle> {
    let stdout = "stdout";
    let console = ConsoleAppender::builder().encoder(Box::new(JsonEncoder::new())).build();

    let config = Config::builder()
        .appender(Appender::builder().build(stdout, Box::new(console)))
        .build(Root::builder().appender(stdout).build(LevelFilter::Info))?;

    Ok(log4rs::init_config(config)?)
}
