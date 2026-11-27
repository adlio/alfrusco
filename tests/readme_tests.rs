//! Keeps the README's Alfred UI example identical to the doctest in
//! `src/ui/mod.rs`, so the README code is compiled and run by `cargo test`.

const README: &str = include_str!("../README.md");
const UI_MODULE: &str = include_str!("../src/ui/mod.rs");

fn readme_example(heading: &str) -> String {
    let section = &README[README.find(heading).expect("README has the section")..];
    let start = section.find("```rust\n").expect("a rust block") + "```rust\n".len();
    let end = start + section[start..].find("```").expect("a closing fence");
    section[start..end].to_string()
}

fn doctest_example(marker: &str) -> String {
    let start = UI_MODULE
        .find(marker)
        .expect("src/ui/mod.rs marks the README example");
    let mut lines = Vec::new();
    for line in UI_MODULE[start..].lines().skip(2) {
        let line = line.strip_prefix("//!").expect("module doc line");
        let line = line.strip_prefix(' ').unwrap_or(line);
        if line == "```" {
            break;
        }
        if !line.starts_with("# ") {
            lines.push(line.to_string());
        }
    }
    lines.join("\n") + "\n"
}

#[test]
fn readme_ui_example_matches_the_doctest() {
    assert_eq!(
        readme_example("### Alfred UI (`ui.json`)"),
        doctest_example("//! <!-- README example -->\n//! ```\n")
    );
}

#[test]
fn readme_backend_example_matches_the_doctest() {
    assert_eq!(
        readme_example("### Alfred UI script backends"),
        doctest_example("//! <!-- README backend example -->\n//! ```no_run\n")
    );
}
