//! The README's example is `examples/pack_list.rs`, and so is the crate documentation's.
use std::fs;

fn read(path: &str) -> String {
    let path = format!("{}/{path}", env!("CARGO_MANIFEST_DIR"));
    fs::read_to_string(path).expect("readable").replace("\r\n", "\n")
}

#[test]
fn the_readme_shows_the_example_file() {
    let example = read("examples/pack_list.rs");
    let readme = read("README.md");
    assert!(
        readme.contains(&format!("```rust\n{example}```")),
        "README.md must hold examples/pack_list.rs in a ```rust block"
    );
}

#[test]
fn the_crate_documentation_shows_the_example_file() {
    let example = read("examples/pack_list.rs");
    let lib = read("src/lib.rs");
    let documented: String = example
        .lines()
        .map(|l| {
            if l.is_empty() {
                "//!\n".to_string()
            } else {
                format!("//! {l}\n")
            }
        })
        .collect();
    assert!(
        lib.contains(&documented),
        "src/lib.rs must hold examples/pack_list.rs as its doc example"
    );
}
