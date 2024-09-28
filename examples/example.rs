// Copyright © 2024 RSS Gen. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT


//! # RSS Gen (rss-gen)
//!
//! This file serves as an entry point for running all the RSS Gen (rss-gen) examples, demonstrating logging levels, formats, macros, and library functionality.

#![allow(missing_docs)]

mod example_data;
mod example_error;
mod example_generator;
mod example_lib;
mod example_macros;
mod example_parser;
mod example_validator;

/// Entry point to run all RSS Gen examples.
///
/// This function calls all the individual examples for log levels, log formats, macros, and library functionality.
fn main() {
    println!("\n🦀 Running RSS Gen (rss-gen) Examples 🦀");

    // Run the example modules.
    let _ = example_data::main();
    let _ = example_error::main();
    example_generator::main();
    example_lib::main();
    let _ = example_macros::main();
    example_parser::main();
    example_validator::main();

    println!("\n🎉 All RustLogs examples completed successfully!\n");
}
