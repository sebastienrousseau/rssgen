#![no_main]

use libfuzzer_sys::fuzz_target;
use rss_gen::data::parse_date;

fuzz_target!(|data: &[u8]| {
    // Convert bytes to string, ignoring invalid UTF-8
    if let Ok(date_str) = std::str::from_utf8(data) {
        // Date parsing should never panic, regardless of input
        let _ = parse_date(date_str);
    }
});