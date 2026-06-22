#![no_main]

use libfuzzer_sys::fuzz_target;
use rss_gen::parser::parse_rss;

fuzz_target!(|data: &[u8]| {
    // Convert bytes to string, ignoring invalid UTF-8
    if let Ok(input_str) = std::str::from_utf8(data) {
        // Try parsing the input as RSS - should never panic
        let _ = parse_rss(input_str, None);
    }
});