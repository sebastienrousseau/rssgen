#![no_main]

use libfuzzer_sys::fuzz_target;
use rss_gen::data::validate_url;

fuzz_target!(|data: &[u8]| {
    // Convert bytes to string, ignoring invalid UTF-8
    if let Ok(url_str) = std::str::from_utf8(data) {
        // URL validation should never panic, regardless of input
        let _ = validate_url(url_str);
    }
});