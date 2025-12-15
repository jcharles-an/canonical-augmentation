use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;


pub fn init_logger(){
    let mut builder = Builder::from_default_env();

    // 2. Set the custom format
    builder.format(|buf, record| {
        // Write the timestamp, level, and message, explicitly leaving out record.module_path()
        writeln!(
            buf,
            "{} [{}] - {}",
            // Use the ISO 8601 compact format for the timestamp
            buf.timestamp_millis(),
            // Level (e.g., INFO, ERROR)
            record.level(),
            // The actual log message
            record.args()
        )
    })
        .filter(None, LevelFilter::Info) // Set the minimum log level
        .init();
}