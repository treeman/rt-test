use anyhow::{anyhow, Result};
use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

mod atm;
mod client;

use atm::Atm;

fn main() {
    if let Err(err) = run() {
        panic!("Found error {:#?}", err)
    }
}

fn run() -> Result<()> {
    let path: PathBuf = input_path()?.into();
    let atm = Atm::from_path(&path)?;
    atm.print_csv()?;
    Ok(())
}

fn input_path() -> anyhow::Result<OsString> {
    match env::args_os().nth(1) {
        None => Err(anyhow!("Please provide an input file")),
        Some(path) => Ok(path),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glob::glob;
    use std::fs;
    use std::path::Path;

    // Quick and dirty test harness that compares input/output files.
    //
    // This will find all '.in' files in the 'test_files' directory,
    // and compare the atm output with the corresponding '.out' file.
    //
    // For example this file:
    //
    //     test_files/base-input.in
    //
    // Will be parsed and the csv output will be compared to the contents of this file:
    //
    //     test_files/base-input.out
    //
    #[test]
    fn diff_input_output_files() {
        for in_path in glob("test_files/*.in")
            .expect("Failed to read glob pattern")
            .filter_map(Result::ok)
        {
            let mut out_path = in_path.clone();
            out_path.set_extension("out");

            assert_output(&in_path, &out_path);
        }
    }

    fn assert_output(in_path: &Path, out_path: &Path) {
        // Note that this holds the contents of both the files in memory (and does a string split
        // and sorts them) so it's not efficient, but it's fine for smaller files.
        let atm = Atm::from_path(in_path).expect(&format!("failed to process {:?}", in_path));
        let got = sort_lines(atm.to_csv_string().expect("failed to write csv string"));
        let expected = sort_lines(
            fs::read_to_string(out_path).expect(&format!("failed to read {:?}", out_path)),
        );
        assert_eq!(got, expected, "failed to match {:?}", in_path);
    }

    fn sort_lines(content: String) -> String {
        let mut lines: Vec<&str> = content.lines().collect();
        lines.sort();
        lines.join("\n")
    }
}
