use souls_core::harness::{
    detect_installed_clis, AgenticCliCommand, AgenticCliKind, AgenticHarnessDispatcher,
};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

#[tokio::test]
async fn test_detect_installed_clis_matrix() {
    let clis = detect_installed_clis().await;
    assert_eq!(clis.len(), 3);

    let agy = clis.iter().find(|c| c.kind == AgenticCliKind::Antigravity).unwrap();
    assert_eq!(agy.binary_name, "agy");
    assert!(agy.supported_modes.contains(&"accept-edits".to_string()));

    // On this host, agy is installed!
    if agy.is_installed {
        assert!(agy.resolved_path.is_some());
        assert!(agy.version.is_some(), "Expected agy version to be detected");
    }

    let claude = clis.iter().find(|c| c.kind == AgenticCliKind::ClaudeCode).unwrap();
    assert_eq!(claude.binary_name, "claude");
    assert!(claude.supported_modes.contains(&"acceptEdits".to_string()));

    let codex = clis.iter().find(|c| c.kind == AgenticCliKind::Codex).unwrap();
    assert_eq!(codex.binary_name, "codex");
    assert!(codex.supported_modes.contains(&"workspace-write".to_string()));
}

#[test]
fn test_build_arguments_for_all_clis() {
    // 1. Antigravity CLI
    let agy_cmd = AgenticCliCommand::new(
        AgenticCliKind::Antigravity,
        "refactor memory module",
        PathBuf::from("Z:\\souls_engine"),
    )
    .with_model("gemini-2.5-pro")
    .with_reasoning_effort("high");

    let agy_args = AgenticHarnessDispatcher::build_arguments(&agy_cmd);
    assert_eq!(agy_args[0], "-p");
    assert_eq!(agy_args[1], "refactor memory module");
    assert!(agy_args.contains(&"--mode=accept-edits".to_string()));
    assert!(agy_args.contains(&"--output-format=stream-json".to_string()));
    assert!(agy_args.contains(&"--dangerously-skip-permissions".to_string()));
    assert!(agy_args.contains(&"--model".to_string()));
    assert!(agy_args.contains(&"gemini-2.5-pro".to_string()));
    assert!(agy_args.contains(&"--effort".to_string()));
    assert!(agy_args.contains(&"high".to_string()));

    // 2. Claude Code CLI
    let claude_cmd = AgenticCliCommand::new(
        AgenticCliKind::ClaudeCode,
        "fix bug in parser",
        PathBuf::from("Z:\\souls_engine"),
    )
    .with_model("claude-3-7-sonnet");

    let claude_args = AgenticHarnessDispatcher::build_arguments(&claude_cmd);
    assert_eq!(claude_args[0], "-p");
    assert_eq!(claude_args[1], "fix bug in parser");
    assert!(claude_args.contains(&"--bare".to_string()));
    assert!(claude_args.contains(&"--permission-mode".to_string()));
    assert!(claude_args.contains(&"acceptEdits".to_string()));
    assert!(claude_args.contains(&"--output-format".to_string()));
    assert!(claude_args.contains(&"stream-json".to_string()));

    // 3. Codex CLI
    let codex_cmd = AgenticCliCommand::new(
        AgenticCliKind::Codex,
        "optimize sql indices",
        PathBuf::from("Z:\\souls_engine"),
    )
    .with_model("o3-mini");

    let codex_args = AgenticHarnessDispatcher::build_arguments(&codex_cmd);
    assert_eq!(codex_args[0], "exec");
    assert_eq!(codex_args[1], "optimize sql indices");
    assert!(codex_args.contains(&"--sandbox".to_string()));
    assert!(codex_args.contains(&"workspace-write".to_string()));
    assert!(codex_args.contains(&"--ephemeral".to_string()));
    assert!(codex_args.contains(&"--json".to_string()));
}

#[tokio::test]
async fn test_git_restore_dirty_files_on_failure() {
    let tmp = tempdir().unwrap();
    let tmp_path = tmp.path();

    // Init local git repo in tempdir
    let init_ok = Command::new("git")
        .arg("init")
        .current_dir(tmp_path)
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !init_ok {
        eprintln!("Skipping git test: git command not available or failed init");
        return;
    }

    // Configure user name and email for commit
    let _ = Command::new("git").args(["config", "user.name", "Test Agent"]).current_dir(tmp_path).status();
    let _ = Command::new("git").args(["config", "user.email", "test@soulsengine.ai"]).current_dir(tmp_path).status();

    // Create and commit a clean file
    let test_file = tmp_path.join("clean.txt");
    {
        let mut f = File::create(&test_file).unwrap();
        f.write_all(b"original clean content\n").unwrap();
    }

    let _ = Command::new("git").args(["add", "clean.txt"]).current_dir(tmp_path).status();
    let _ = Command::new("git").args(["commit", "-m", "initial commit"]).current_dir(tmp_path).status();

    // Mutate the file (simulate broken/failed CLI edit)
    {
        let mut f = File::create(&test_file).unwrap();
        f.write_all(b"corrupted bad edit\n").unwrap();
    }

    // Verify git status sees it as dirty
    let dirty_list = AgenticHarnessDispatcher::git_restore_dirty_files(tmp_path).await;
    assert_eq!(dirty_list.len(), 1);
    assert!(dirty_list[0].contains("clean.txt"));

    // Verify content was restored to original
    let restored_content = std::fs::read_to_string(&test_file).unwrap();
    assert_eq!(restored_content.trim(), "original clean content");
}
