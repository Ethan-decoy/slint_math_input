//! Native mathematical input demonstration.
#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]
mod support;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    support::run()
}
