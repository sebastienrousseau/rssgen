// Copyright © 2024 RSS Gen. All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
// See LICENSE-APACHE.md and LICENSE-MIT.md in the repository root for full license information.

//! # RSS Gen Parser Usage Examples
//!
//! This example demonstrates how to parse RSS feeds from XML content using the RSS Gen library.

#![allow(missing_docs)]

use rss_gen::parse_rss;

/// Entry point for the RSS Gen parser examples.
///
/// This function demonstrates parsing XML content into structured RSS data.
pub(crate) fn main() {
    println!("\n🧪 RSS Gen Parser Usage Examples \n");

    // Run the example for RSS feed parsing
    parse_rss_example();

    println!("\n🎉  All examples completed successfully!\n");
}

/// Demonstrates parsing an RSS 2.0 feed from XML content.
fn parse_rss_example() {
    println!("🦀  Parse Rss 2.0 Feed Example");
    println!("---------------------------------------------");

    let xml_content = r#"
        <?xml version="1.0" encoding="utf-8"?>
        <rss version="2.0" xmlns:p="http://world.episerver.com/feeds/EpiserverRSS20.xsd">
          <channel>
            <title>Rss 2.0 Sample customer product catalog feed</title>
            <link>http://www.yourdomain.com</link>
            <description>The latest product catalog feed of sample customer.</description>
            <item>
              <title>Sample Coat 001</title>
              <link>http://yourdomain.com/c001.aspx</link>
              <guid>c001</guid>
              <p:imageLink>htto://yourdomain.com/image/c001.jpg</p:imageLink>
              <pubDate>Sun, 20 Apr 2008 00:00:00 GMT</pubDate>
              <description>High quality wool coat.</description>
              <category><![CDATA[Coats>Winter Wear]]></category>
              <category><![CDATA[Coats>Cashmere]]></category>
              <p:brand>SampleManufactor1</p:brand>
              <p:inStock>Y</p:inStock>
              <p:stock>20</p:stock>
              <p:recommend>Y</p:recommend>
              <p:tags>short sleeve,blue,men’s,outdoor</p:tags>
              <p:recommended>h001,h003</p:recommended>
              <p:attribute name="Colour">Black,Grey</p:attribute>
              <p:attribute name="Size">10, 12, 14</p:attribute>
              <p:price>
                <p:unitPrice>10</p:unitPrice>
                <p:salePrice>8.50</p:salePrice>
                <p:currency>GBP</p:currency>
              </p:price>
            </item>
            <item>
              <title>Sample Hat 002</title>
              <link>http://yourdomain.com/h002.aspx</link>
              <guid>h002</guid>
              <p:imageLink>http://yourdomain.com/image/h002.jpg</p:imageLink>
              <pubDate>Wed, 30 Apr 2008 00:00:00 GMT</pubDate>
              <description>Low quality wool hat.</description>
              <category><![CDATA[Hats>Wool Hat]]></category>
              <p:brand>SampleManufactor2</p:brand>
              <p:inStock>Y</p:inStock>
              <p:stock>20</p:stock>
              <p:recommend>Y</p:recommend>
              <p:tags>short sleeve,blue,men’s,outdoor</p:tags>
              <p:recommended>h001,h003</p:recommended>
              <p:attribute name="Colour">Black </p:attribute>
              <p:attribute name="Size">16,18,20</p:attribute>
              <p:price>
                <p:unitPrice>5</p:unitPrice>
                <p:currency>GBP</p:currency>
              </p:price>
            </item>
          </channel>
        </rss>
    "#;

    // Parse the RSS content
    match parse_rss(xml_content) {
        Ok(parsed_data) => {
            // Pretty-print the entire parsed data using the Debug trait
            println!(
                "    ✅  Parsed RSS feed data: {:#?}",
                parsed_data
            );

            // Directly access individual fields of parsed_data
            println!(
                "    ✅  Parsed RSS feed title: {:?}",
                parsed_data.title
            );
            println!(
                "    ✅  Parsed RSS feed link: {:?}",
                parsed_data.link
            );
            println!(
                "    ✅  Parsed RSS feed description: {:?}",
                parsed_data.description
            );
            println!(
                "    ✅  Number of items: {}",
                parsed_data.items.len()
            );

            // Print details of the first item, if available
            if let Some(first_item) = parsed_data.items.first() {
                println!(
                    "    ✅  First item title: {:?}",
                    first_item.title
                );
                println!(
                    "    ✅  First item link: {:?}",
                    first_item.link
                );
                println!(
                    "    ✅  First item description: {:?}",
                    first_item.description
                );
            }
        }
        Err(e) => println!("    ❌  Error parsing RSS feed: {}", e),
    }
}
