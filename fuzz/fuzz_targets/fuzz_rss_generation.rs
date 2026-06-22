#![no_main]

use libfuzzer_sys::fuzz_target;
use rss_gen::{data::{RssData, RssItem, RssVersion}, generate_rss};
use arbitrary::Arbitrary;

#[derive(Arbitrary, Debug)]
struct FuzzRssData {
    version: FuzzRssVersion,
    title: String,
    link: String,
    description: String,
    author: String,
    category: String,
}

#[derive(Arbitrary, Debug)]
enum FuzzRssVersion {
    RSS0_90,
    RSS0_91,
    RSS0_92,
    RSS1_0,
    RSS2_0,
}

impl From<FuzzRssVersion> for RssVersion {
    fn from(fuzz_version: FuzzRssVersion) -> Self {
        match fuzz_version {
            FuzzRssVersion::RSS0_90 => RssVersion::RSS0_90,
            FuzzRssVersion::RSS0_91 => RssVersion::RSS0_91,
            FuzzRssVersion::RSS0_92 => RssVersion::RSS0_92,
            FuzzRssVersion::RSS1_0 => RssVersion::RSS1_0,
            FuzzRssVersion::RSS2_0 => RssVersion::RSS2_0,
        }
    }
}

fuzz_target!(|fuzz_data: FuzzRssData| {
    // Limit string sizes to prevent memory exhaustion
    let title = if fuzz_data.title.len() > 1000 {
        fuzz_data.title.chars().take(1000).collect()
    } else {
        fuzz_data.title
    };
    let link = if fuzz_data.link.len() > 2000 {
        fuzz_data.link.chars().take(2000).collect()
    } else {
        fuzz_data.link
    };
    let description = if fuzz_data.description.len() > 10000 {
        fuzz_data.description.chars().take(10000).collect()
    } else {
        fuzz_data.description
    };
    let author = if fuzz_data.author.len() > 1000 {
        fuzz_data.author.chars().take(1000).collect()
    } else {
        fuzz_data.author
    };
    let category = if fuzz_data.category.len() > 1000 {
        fuzz_data.category.chars().take(1000).collect()
    } else {
        fuzz_data.category
    };

    let rss_data = RssData::new(Some(fuzz_data.version.into()))
        .title(title)
        .link(link)
        .description(description)
        .author(author)
        .category(category);

    // Should never panic, regardless of input
    let _ = generate_rss(&rss_data);
    let _ = rss_data.validate();
    let _ = rss_data.validate_size();
});