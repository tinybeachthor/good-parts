use std::fs;
use std::path::Path;

use assert_cmd::Command;
use tempfile::TempDir;

const SIMPLE: &str = "fn add(a: i32, b: i32) -> i32 {\n    if a > 0 { a + b } else { b }\n}\n";

const TANGLED: &str = r#"
fn tangled(xs: &[i32]) -> i32 {
    let mut total = 0;
    for x in xs {
        if *x > 0 {
            if *x % 2 == 0 {
                total += x;
            } else if *x % 3 == 0 {
                total -= x;
            }
        } else {
            while total > 100 {
                total /= 2;
            }
        }
    }
    total
}
"#;

fn fixture() -> TempDir {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join("src")).unwrap();
    fs::write(dir.path().join("src/simple.rs"), SIMPLE).unwrap();
    fs::write(dir.path().join("src/tangled.rs"), TANGLED).unwrap();
    fs::write(dir.path().join("notes.txt"), "fn not_code() {}").unwrap();
    dir
}

fn run(bin: &str, args: &[&str], path: &Path) -> String {
    let output = Command::cargo_bin(bin)
        .unwrap()
        .args(args)
        .arg(path)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn ranks_most_complex_function_first() {
    let dir = fixture();
    let stdout = run("good-parts", &[], dir.path());
    let first_row = stdout.lines().nth(1).unwrap();
    assert!(first_row.contains("tangled.rs:2"), "{stdout}");
    assert!(stdout.contains("add"), "{stdout}");
    assert!(!stdout.contains("not_code"), "{stdout}");
}

#[test]
fn json_output_is_parseable() {
    let dir = fixture();
    let stdout = run(
        "good-parts",
        &["--format", "json", "--threshold", "5"],
        dir.path(),
    );
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["summary"]["files"], 2);
    assert_eq!(json["summary"]["over_threshold"], 1);
    assert_eq!(json["hotspots"][0]["name"], "tangled");
    assert_eq!(json["hotspots"][0]["exceeds_threshold"], true);
}

#[test]
fn file_ranking_and_exclude() {
    let dir = fixture();
    let stdout = run(
        "good-parts",
        &["--files", "--exclude", "**/tangled.rs"],
        dir.path(),
    );
    assert!(stdout.contains("simple.rs"), "{stdout}");
    assert!(!stdout.contains("tangled.rs"), "{stdout}");
}

#[test]
fn cargo_subcommand_matches_standalone() {
    let dir = fixture();
    assert_eq!(
        run("cargo-good-parts", &["good-parts"], dir.path()),
        run("good-parts", &[], dir.path()),
    );
}

#[test]
fn explicit_paths_are_combined_without_duplicates() {
    let dir = fixture();
    let simple = dir.path().join("src/simple.rs");
    let json = |args: &[&Path]| {
        let mut cmd = Command::cargo_bin("good-parts").unwrap();
        cmd.args(["--format", "json", "--min-cognitive", "0"])
            .args(args);
        let output = cmd.output().unwrap();
        assert!(output.status.success(), "{output:?}");
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap()
    };

    let only_simple = json(&[&simple]);
    assert_eq!(only_simple["summary"]["files"], 1);
    assert_eq!(only_simple["hotspots"][0]["name"], "add");

    let overlapping = json(&[dir.path(), &simple]);
    assert_eq!(overlapping["summary"]["files"], 2);
    assert_eq!(overlapping["summary"]["functions"], 2);
}

#[test]
fn json_output_is_versioned_and_stable() {
    let dir = fixture();
    let stdout = run("good-parts", &["--format", "json"], dir.path());
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["kind"], "functions");
    let top = &json["hotspots"][0];
    let expected = format!("{}:{}", top["file"].as_str().unwrap(), top["start_line"]);
    assert_eq!(top["location"], expected.as_str());
    let score = top["score"].as_f64().unwrap();
    assert_eq!(score, (score * 100.0).round() / 100.0);

    let files = run("good-parts", &["--format", "json", "--files"], dir.path());
    let files: serde_json::Value = serde_json::from_str(&files).unwrap();
    assert_eq!(files["kind"], "files");
}

fn exit_code(args: &[&str], path: &Path) -> i32 {
    let output = Command::cargo_bin("good-parts")
        .unwrap()
        .args(args)
        .arg(path)
        .output()
        .unwrap();
    output.status.code().unwrap()
}

#[test]
fn exit_codes() {
    let dir = fixture();
    let gate = ["--fail-over-threshold", "--threshold"];
    assert_eq!(exit_code(&[gate[0], gate[1], "5"], dir.path()), 1);
    assert_eq!(exit_code(&[gate[0], gate[1], "50"], dir.path()), 0);
    assert_eq!(exit_code(&["--threshold", "5"], dir.path()), 0);
    assert_eq!(exit_code(&[], &dir.path().join("missing")), 2);
}

#[test]
fn long_help_has_examples_and_exit_codes() {
    let long = Command::cargo_bin("good-parts")
        .unwrap()
        .arg("--help")
        .output()
        .unwrap();
    let long = String::from_utf8(long.stdout).unwrap();
    assert!(long.contains("Examples:"), "{long}");
    assert!(long.contains("Exit codes:"), "{long}");

    let short = Command::cargo_bin("good-parts")
        .unwrap()
        .arg("-h")
        .output()
        .unwrap();
    let short = String::from_utf8(short.stdout).unwrap();
    assert!(!short.contains("Examples:"), "{short}");
}
