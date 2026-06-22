#![no_main]

use libfuzzer_sys::fuzz_target;
use rss_gen::generator::sanitize_content;

fuzz_target!(|data: &[u8]| {
    // Convert bytes to string, ignoring invalid UTF-8
    if let Ok(content_str) = std::str::from_utf8(data) {
        // Content sanitization should never panic
        let sanitized = sanitize_content(content_str);

        // Additional invariant checks
        assert!(!sanitized.contains('<') || sanitized.contains("&lt;"));
        assert!(!sanitized.contains('>') || sanitized.contains("&gt;"));
        assert!(!sanitized.contains('&') || sanitized.contains("&amp;"));
    }
});