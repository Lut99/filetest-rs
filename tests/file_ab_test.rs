//  FILE A/B TEST.rs
//    by Lut99
//
//  Description:
//!   File showcasing the usage of the A/B test macro.
//

use std::path::PathBuf;

use filetest::file_ab_test;


/***** ENTRYPOINT *****/
/// Test that is run for all pairs of files with the given suffixes.
///
/// That is, the macro will find all files in the `files`-directory ending in `.txt`. Then, for
/// each of those, it finds a file with the same basename but ending in `.gold.txt`; if found, it
/// generates a `#[test]`-case for those and calls your function for those instances.
///
/// In this case, when running this test, you should see three cases: `test_lowercase_hello_world`;
/// `test_lowercase_uppercase`; and `test_lowercase_lowercase`.
#[file_ab_test(input = ".txt", gold = ".gold.txt", path = concat!(env!("CARGO_MANIFEST_DIR"), "/../filetest/tests/files"))]
fn test_lowercase(input_path: PathBuf, input: Vec<u8>, _gold_path: PathBuf, gold: Vec<u8>) {
    // Interpret both as strings
    let input: &str = std::str::from_utf8(&input).unwrap();
    let gold: &str = std::str::from_utf8(&gold).unwrap();

    // Run the test on the input file!
    let pred: String = input.to_lowercase();

    // Check if they match
    if pred != gold {
        panic!("Test failed for input file {input_path:?}: prediction does not match gold value");
    }
}
