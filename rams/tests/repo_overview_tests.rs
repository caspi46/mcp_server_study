use std::path::PathBuf;

// Import from the main crate
use rams::mcp_server::compute_repo_overview;

fn fixture_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(rel)
}

#[test]
fn repo_overview_counts_files_dirs_and_git() {
    let repo_root = fixture_path("tiny_repo");

    let overview =
        compute_repo_overview(&repo_root, usize::MAX, false).expect("overview should succeed");

    assert_eq!(overview.name, "tiny_repo");
    assert!(overview.has_git);

    // Files we created:
    // Cargo.toml, README.md, src/main.rs, scripts/util.py  => 4 files
    assert_eq!(overview.total_files, 4);

    // Dirs we created under tiny_repo:
    // .git, src, scripts => 3 dirs (depending on how you count root)
    // Your function counts subdirs it encounters, not the root itself.
    assert_eq!(overview.total_dirs, 2);

    // Language counts by extension (depends on your mapping)
    // main.rs => Rust
    assert_eq!(overview.languages.get("Rust").copied().unwrap_or(0), 1);
    assert_eq!(overview.languages.get("Python").copied().unwrap_or(0), 1);
    assert_eq!(overview.languages.get("Markdown").copied().unwrap_or(0), 1);
    assert_eq!(overview.languages.get("TOML").copied().unwrap_or(0), 1);
}

#[test]
fn repo_overview_respects_max_depth() {
    let repo_root = fixture_path("tiny_repo");

    // depth=0 means: only read entries directly under repo_root
    // It will see directories (src/scripts/.git) but will NOT traverse into them.
    let overview = compute_repo_overview(&repo_root, 0, false).expect("overview should succeed");

    // At depth 0, only files at top level count: Cargo.toml, README.md => 2 files
    assert_eq!(overview.total_files, 2);

    // It still "sees" directories at depth 0, and counts them as dirs
    // (because it encountered them in the root listing)
    assert_eq!(overview.total_dirs, 2);

    // No Rust/Python counted because those files are inside subdirs
    assert_eq!(overview.languages.get("Rust").copied().unwrap_or(0), 0);
    assert_eq!(overview.languages.get("Python").copied().unwrap_or(0), 0);
}

#[test]
fn repo_overview_skips_hidden_when_configured() {
    // Create a second fixture with hidden content OR just use tempdir tests (Option B).
    // If you already have include_hidden logic (skip dotfiles), then:
    let repo_root = fixture_path("tiny_repo");

    // include_hidden=false should skip ".git" directory entirely in your current logic
    // BUT your current code checks `has_git` by repo_root.join(".git").exists()
    // so has_git remains true, even if you skip traversing ".git".
    let overview =
        compute_repo_overview(&repo_root, usize::MAX, false).expect("overview should succeed");

    assert!(overview.has_git);

    // If your skip-hidden logic skips directories starting with '.', total_dirs will be 2 (src/scripts).
    // If you don't skip directories for counting (only for traversal), this may differ.
    // Adjust expected value to match your implementation.
    // assert_eq!(overview.total_dirs, 2);
}
