// Copyright © 2024 RSS Gen. All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
// See LICENSE-APACHE.md and LICENSE-MIT.md in the repository root for full license information.

//! # RSS Gen Parser Usage Examples
//!
//! This example demonstrates how to parse RSS feeds from XML content using the RSS Gen library.

#![allow(missing_docs)]

use rss_gen::parse_rss;
use std::error::Error;

/// Custom error type for example execution
#[derive(Debug)]
struct ExampleError {
    message: String,
}

impl std::fmt::Display for ExampleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Example Error: {}", self.message)
    }
}

impl Error for ExampleError {}

/// Entry point for the RSS Gen parser examples.
///
/// This function demonstrates parsing XML content into structured RSS data.
pub fn main() -> Result<(), Box<dyn Error>> {
    println!("\n🧪 RSS Gen Parser Usage Examples \n");

    // Run the example for RSS feed parsing
    parse_rss_example()?;
    parse_rss_0_90_example()?;
    parse_rss_0_91_example()?;
    parse_rss_0_92_example()?;
    parse_rss_1_0_example()?;
    parse_rss_2_0_example()?;

    println!("\n🎉  All examples completed successfully!\n");
    Ok(())
}

/// Demonstrates parsing an RSS 2.0 feed from XML content.
fn parse_rss_example() -> Result<(), Box<dyn Error>> {
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
              <p:tags>short sleeve,blue,men's,outdoor</p:tags>
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
              <p:tags>short sleeve,blue,men's,outdoor</p:tags>
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
    let parsed_data = parse_rss(xml_content).map_err(|e| {
        Box::new(ExampleError {
            message: format!("Failed to parse RSS feed: {}", e),
        }) as Box<dyn Error>
    })?;

    // Pretty-print the entire parsed data using the Debug trait
    println!("    ✅  Parsed RSS feed data: {:#?}", parsed_data);

    // Directly access individual fields of parsed_data
    println!("    ✅  Parsed RSS feed title: {:?}", parsed_data.title);
    println!("    ✅  Parsed RSS feed link: {:?}", parsed_data.link);
    println!(
        "    ✅  Parsed RSS feed description: {:?}",
        parsed_data.description
    );
    println!("    ✅  Number of items: {}", parsed_data.items.len());

    // Print details of the first item, if available
    if let Some(first_item) = parsed_data.items.first() {
        println!("    ✅  First item title: {:?}", first_item.title);
        println!("    ✅  First item link: {:?}", first_item.link);
        println!(
            "    ✅  First item description: {:?}",
            first_item.description
        );
    }

    Ok(())
}

fn parse_rss_0_90_example() -> Result<(), Box<dyn Error>> {
    println!("\n🦀 Parsing RSS 0.90 Example");
    let rss_0_90_content = r#"
    <?xml version="1.0"?>
    <rdf:RDF
        xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
        xmlns="http://channel.netscape.com/rdf/simple/0.9/">
        <channel>
            <title>Mozilla Dot Org</title>
            <link>http://www.mozilla.org</link>
            <description>the Mozilla Organization web site</description>
        </channel>
        <image>
            <title>Mozilla</title>
            <url>http://www.mozilla.org/images/moz.gif</url>
            <link>http://www.mozilla.org</link>
        </image>
        <item>
            <title>New Status Updates</title>
            <link>http://www.mozilla.org/status/</link>
        </item>
        <item>
            <title>Bugzilla Reorganized</title>
            <link>http://www.mozilla.org/bugs/</link>
        </item>
    </rdf:RDF>
    "#;

    let parsed_data = parse_rss(rss_0_90_content)?;
    println!("Parsed RSS 0.90 feed:");
    println!("Title: {}", parsed_data.title);
    println!("Link: {}", parsed_data.link);
    println!("Description: {}", parsed_data.description);
    println!("Number of items: {}", parsed_data.items.len());
    Ok(())
}

fn parse_rss_0_91_example() -> Result<(), Box<dyn Error>> {
    println!("\n🦀 Parsing RSS 0.91 Example");
    let rss_0_91_content = r#"
    <?xml version="1.0" encoding="ISO-8859-1"?>
    <rss version="0.91">
        <channel>
            <title>XML.com</title>
            <link>http://www.xml.com/</link>
            <description>XML.com features a rich mix of information and services for the XML community.</description>
            <language>en-us</language>
            <item>
                <title>Normalizing XML, Part 2</title>
                <link>http://www.xml.com/pub/a/2002/12/04/normalizing.html</link>
                <description>In this second and final look at applying relational normalization
                    techniques to W3C XML Schema data modeling, Will Provost discusses when not to
                    normalize, the scope of uniqueness and the fourth and fifth normal forms.</description>
            </item>
        </channel>
    </rss>
    "#;

    let parsed_data = parse_rss(rss_0_91_content)?;
    println!("Parsed RSS 0.91 feed:");
    println!("Title: {}", parsed_data.title);
    println!("Link: {}", parsed_data.link);
    println!("Description: {}", parsed_data.description);
    println!("Language: {}", parsed_data.language);
    println!("Number of items: {}", parsed_data.items.len());
    Ok(())
}

fn parse_rss_0_92_example() -> Result<(), Box<dyn Error>> {
    println!("\n🦀 Parsing RSS 0.92 Example");
    let rss_0_92_content = r#"
    <?xml version="1.0"?>
    <rss version="0.92">
        <channel>
            <title>My Website</title>
            <link>http://www.example.com/</link>
            <description>News and updates from my website.</description>
            <language>en-us</language>
            <lastBuildDate>Mon, 11 Oct 2024 21:57:00 GMT</lastBuildDate>
            <image>
                <title>My Website Image</title>
                <url>http://www.example.com/image.jpg</url>
                <link>http://www.example.com/</link>
            </image>
            <item>
                <title>First article title</title>
                <link>http://www.example.com/article1</link>
                <description>Short description of the article.</description>
                <pubDate>Mon, 11 Oct 2024 12:00:00 GMT</pubDate>
            </item>
        </channel>
    </rss>
    "#;

    let parsed_data = parse_rss(rss_0_92_content)?;
    println!("Parsed RSS 0.92 feed:");
    println!("Title: {}", parsed_data.title);
    println!("Link: {}", parsed_data.link);
    println!("Description: {}", parsed_data.description);
    println!("Last Build Date: {}", parsed_data.last_build_date);
    println!("Number of items: {}", parsed_data.items.len());
    Ok(())
}

fn parse_rss_1_0_example() -> Result<(), Box<dyn Error>> {
    println!("\n🦀 Parsing RSS 1.0 Example");
    let rss_1_0_content = r#"
    <?xml version="1.0"?>
    <rdf:RDF
        xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
        xmlns="http://purl.org/rss/1.0/"
        xmlns:dc="http://purl.org/dc/elements/1.1/"
    >
        <channel rdf:about="http://www.xml.com/cs/xml/query/q/19">
            <title>XML.com</title>
            <link>http://www.xml.com/</link>
            <description>XML.com features a rich mix of information and services for the XML community.</description>
            <language>en-us</language>
            <items>
                <rdf:Seq>
                    <rdf:li rdf:resource="http://www.xml.com/pub/a/2002/12/04/normalizing.html" />
                    <rdf:li rdf:resource="http://www.xml.com/pub/a/2002/12/04/som.html" />
                    <rdf:li rdf:resource="http://www.xml.com/pub/a/2002/12/04/svg.html" />
                </rdf:Seq>
            </items>
        </channel>
        <item rdf:about="http://www.xml.com/pub/a/2002/12/04/normalizing.html">
            <title>Normalizing XML, Part 2</title>
            <link>http://www.xml.com/pub/a/2002/12/04/normalizing.html</link>
            <description>In this second and final look at applying relational normalization techniques
                to W3C XML Schema data modeling, Will Provost discusses when not to normalize, the scope
                of uniqueness and the fourth and fifth normal forms.</description>
            <dc:creator>Will Provost</dc:creator>
            <dc:date>2002-12-04</dc:date>
        </item>
    </rdf:RDF>
    "#;

    let parsed_data = parse_rss(rss_1_0_content)?;
    println!("Parsed RSS 1.0 feed:");
    println!("Title: {}", parsed_data.title);
    println!("Link: {}", parsed_data.link);
    println!("Description: {}", parsed_data.description);
    println!("Language: {}", parsed_data.language);
    println!("Number of items: {}", parsed_data.items.len());
    Ok(())
}

fn parse_rss_2_0_example() -> Result<(), Box<dyn Error>> {
    println!("\n🦀 Parsing RSS 2.0 Example");
    let rss_2_0_content = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <rss version="2.0" xmlns:dc="http://purl.org/dc/elements/1.1/">
        <channel>
            <title>XML.com</title>
            <link>http://www.xml.com/</link>
            <description>XML.com features a rich mix of information and services for the XML community.</description>
            <language>en-us</language>
            <item>
                <title>Normalizing XML, Part 2</title>
                <link>http://www.xml.com/pub/a/2002/12/04/normalizing.html</link>
                <description>In this second and final look at applying relational normalization
                    techniques to W3C XML Schema data modeling, Will Provost discusses when not to
                    normalize, the scope of uniqueness and the fourth and fifth normal forms.</description>
                <dc:creator>Will Provost</dc:creator>
                <dc:date>2002-12-04</dc:date>
            </item>
        </channel>
    </rss>
    "#;

    let parsed_data = parse_rss(rss_2_0_content)?;
    println!("Parsed RSS 2.0 feed:");
    println!("Title: {}", parsed_data.title);
    println!("Link: {}", parsed_data.link);
    println!("Description: {}", parsed_data.description);
    println!("Language: {}", parsed_data.language);
    println!("Number of items: {}", parsed_data.items.len());
    Ok(())
}
