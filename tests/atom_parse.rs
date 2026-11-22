// Copyright © 2024 RSS Gen. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Regression tests for parsing Atom 1.0 feeds through `parse_rss`
//! (issue #85).

use rss_gen::parse_rss;

/// The example feed from RFC 4287 §1.1, verbatim as reported in #85.
const RFC4287_EXAMPLE: &str = r#"
<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">

  <title>Example Feed</title>
  <link href="http://example.org/"/>
  <updated>2003-12-13T18:30:02Z</updated>
  <author>
    <name>John Doe</name>
  </author>
  <id>urn:uuid:60a76c80-d399-11d9-b93C-0003939e0af6</id>

  <entry>
    <title>Atom-Powered Robots Run Amok</title>
    <link href="http://example.org/2003/12/13/atom03"/>
    <id>urn:uuid:1225c695-cfb8-4ebb-aaaa-80da344efa6a</id>
    <updated>2003-12-13T18:30:02Z</updated>
    <summary>Some text.</summary>
  </entry>
</feed>
"#;

#[test]
fn parses_rfc4287_example_feed() {
    let feed = parse_rss(RFC4287_EXAMPLE, None)
        .expect("the RFC 4287 example feed must parse");

    assert_eq!(feed.title, "Example Feed");
    assert_eq!(feed.link, "http://example.org/");
    assert_eq!(feed.last_build_date, "2003-12-13T18:30:02Z");
    assert_eq!(feed.author, "John Doe");
    assert_eq!(
        feed.guid,
        "urn:uuid:60a76c80-d399-11d9-b93C-0003939e0af6"
    );

    assert_eq!(feed.items.len(), 1);
    let entry = &feed.items[0];
    assert_eq!(entry.title, "Atom-Powered Robots Run Amok");
    assert_eq!(entry.link, "http://example.org/2003/12/13/atom03");
    assert_eq!(
        entry.guid,
        "urn:uuid:1225c695-cfb8-4ebb-aaaa-80da344efa6a"
    );
    assert_eq!(entry.pub_date, "2003-12-13T18:30:02Z");
    assert_eq!(entry.description, "Some text.");
}

#[test]
fn maps_optional_atom_elements() {
    let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom"
      xmlns:ext="http://example.org/ext">
  <title type="text">Feed</title>
  <subtitle>All about feeds</subtitle>
  <link rel="self" href="http://example.org/feed.atom"/>
  <link rel="alternate" href="http://example.org/"/>
  <link rel="alternate" href="http://example.org/second"/>
  <rights>(c) 2003 Example</rights>
  <generator uri="http://example.org/gen">Gen</generator>
  <category term="robots"/>
  <logo>http://example.org/logo.png</logo>
  <ext:unknown>ignored extension element</ext:unknown>
  <id>urn:feed</id>
  <updated>2003-12-13T18:30:02Z</updated>
  <entry>
    <title>Entry</title>
    <link rel="enclosure" href="http://example.org/a.mp3"/>
    <link href="http://example.org/entry"/>
    <id>urn:entry</id>
    <updated>2003-12-14T00:00:00Z</updated>
    <published>2003-12-13T08:29:29-04:00</published>
    <author><name>Jane</name><email>jane@example.org</email></author>
    <category term="news"/>
    <content type="xhtml"><div xmlns="http://www.w3.org/1999/xhtml">Body <b>text</b></div></content>
    <ext:rating>5</ext:rating>
  </entry>
  <entry>
    <title>Second</title>
    <id>urn:entry2</id>
    <updated>2003-12-15T00:00:00Z</updated>
    <summary>Short</summary>
    <content>Long</content>
  </entry>
</feed>"#;

    let feed = parse_rss(xml, None).expect("feed must parse");
    assert_eq!(feed.title, "Feed");
    assert_eq!(feed.description, "All about feeds");
    assert_eq!(feed.atom_link, "http://example.org/feed.atom");
    assert_eq!(feed.link, "http://example.org/");
    assert_eq!(feed.copyright, "(c) 2003 Example");
    assert_eq!(feed.generator, "Gen");
    assert_eq!(feed.category, "robots");
    assert_eq!(feed.image_url, "http://example.org/logo.png");
    assert_eq!(feed.author, "");

    assert_eq!(feed.items.len(), 2);
    let first = &feed.items[0];
    assert_eq!(first.link, "http://example.org/entry");
    assert_eq!(first.pub_date, "2003-12-13T08:29:29-04:00");
    assert_eq!(first.author, "Jane");
    assert_eq!(first.category.as_deref(), Some("news"));
    assert_eq!(first.description, "Body text");

    let second = &feed.items[1];
    assert_eq!(second.pub_date, "2003-12-15T00:00:00Z");
    assert_eq!(second.description, "Short");
}

#[test]
fn feed_without_atom_namespace_is_still_rejected() {
    let xml = r#"<feed><title>Not Atom</title></feed>"#;
    assert!(parse_rss(xml, None).is_err());
}

#[test]
fn entry_source_metadata_does_not_overwrite_entry() {
    let xml = r#"<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Feed</title>
  <entry>
    <title>Entry</title>
    <id>urn:entry</id>
    <source>
      <title>Original Feed</title>
      <id>urn:original</id>
      <link href="http://original.example.org/"/>
      <author><name>Someone Else</name></author>
    </source>
    <author><name>Jane</name></author>
  </entry>
</feed>"#;
    let feed = parse_rss(xml, None).expect("feed must parse");
    let entry = &feed.items[0];
    assert_eq!(entry.title, "Entry");
    assert_eq!(entry.guid, "urn:entry");
    assert_eq!(entry.link, "");
    assert_eq!(entry.author, "Jane");
    assert_eq!(feed.title, "Feed");
}
