// Copyright © 2024 RSS Gen. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # RSS Gen (rss-gen) Examples
//!
//! This module serves as an entry point for running all the RSS Gen (rss-gen) examples,
//! demonstrating various aspects of the library including logging levels, formats,
//! macros, and core functionality.

mod example_data;
mod example_error;
mod example_generator;
mod example_lib;
mod example_macros;
mod example_parser;
mod example_validator;

use std::error::Error;

/// Runs all RSS Gen examples.
///
/// This function sequentially executes all individual examples, demonstrating
/// various features and capabilities of the RSS Gen library.
fn main() -> Result<(), Box<dyn Error>> {
    println!("\n🦀 Running RSS Gen (rss-gen) Examples 🦀");

    // Run the example modules
    example_data::main()?;
    example_error::main()?;
    example_generator::main()?;
    example_lib::main()?;
    example_macros::main()?;
    example_parser::main()?;
    example_validator::main()?;

    println!("\n🎉 All RSS Gen examples completed successfully!\n");
    Ok(())
}
