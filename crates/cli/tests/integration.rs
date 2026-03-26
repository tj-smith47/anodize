use std::process::Command;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use std::fs;

/// Helper to create a minimal Cargo project for testing
fn create_test_project(dir: &std::path::Path) {
    // Create Cargo.toml
    fs::write(dir.join("Cargo.toml"), r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2024"

[[bin]]
name = "test-project"
path = "src/main.rs"
"#).unwrap();

    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/main.rs"), r#"fn main() { println!("hello"); }"#).unwrap();
}

/// Helper to create an anodize.yaml config
fn create_config(dir: &std::path::Path, content: &str) {
    fs::write(dir.join(".anodize.yaml"), content).unwrap();
}

/// Helper to init git repo with a tag
fn init_git_repo(dir: &std::path::Path) {
    let run = |args: &[&str]| {
        Command::new("git")
            .args(args)
            .current_dir(dir)
            .output()
            .expect("git command failed");
    };
    run(&["init"]);
    run(&["config", "user.email", "test@test.com"]);
    run(&["config", "user.name", "Test"]);
    run(&["add", "-A"]);
    run(&["commit", "-m", "initial"]);
    run(&["tag", "v0.1.0"]);
}

#[test]
fn test_check_valid_config() {
    let tmp = TempDir::new().unwrap();
    create_test_project(tmp.path());
    create_config(tmp.path(), r#"
project_name: test-project
crates:
  - name: test-project
    path: "."
    tag_template: "v{{ .Version }}"
"#);

    let output = Command::new(env!("CARGO_BIN_EXE_anodize"))
        .arg("check")
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(output.status.success(), "check should succeed: {}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn test_check_invalid_config() {
    let tmp = TempDir::new().unwrap();
    // No anodize.yaml at all
    let output = Command::new(env!("CARGO_BIN_EXE_anodize"))
        .arg("check")
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("no anodize config file found"));
}

#[test]
fn test_init_generates_config() {
    let tmp = TempDir::new().unwrap();
    create_test_project(tmp.path());
    init_git_repo(tmp.path());

    let output = Command::new(env!("CARGO_BIN_EXE_anodize"))
        .arg("init")
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(output.status.success(), "init should succeed: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("project_name:"));
    assert!(stdout.contains("test-project"));
    assert!(stdout.contains("tag_template:"));
}

#[test]
fn test_help_output() {
    let output = Command::new(env!("CARGO_BIN_EXE_anodize"))
        .arg("--help")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("release"));
    assert!(stdout.contains("build"));
    assert!(stdout.contains("check"));
    assert!(stdout.contains("init"));
    assert!(stdout.contains("changelog"));
    assert!(stdout.contains("completion"), "help should list completion command");
    assert!(stdout.contains("healthcheck"), "help should list healthcheck command");
}

#[test]
fn test_version_output() {
    let output = Command::new(env!("CARGO_BIN_EXE_anodize"))
        .arg("--version")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("anodize"));
}

#[test]
fn test_check_with_config_flag() {
    let tmp = TempDir::new().unwrap();
    create_test_project(tmp.path());

    // Place config at a non-default path
    let custom_dir = tmp.path().join("configs");
    fs::create_dir_all(&custom_dir).unwrap();
    let config_path = custom_dir.join("release.yaml");
    fs::write(&config_path, r#"
project_name: test-project
crates:
  - name: test-project
    path: "."
    tag_template: "v{{ .Version }}"
"#).unwrap();

    // Use -f to point to the custom config
    let output = Command::new(env!("CARGO_BIN_EXE_anodize"))
        .args(["-f", config_path.to_str().unwrap(), "check"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "check -f should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_check_with_config_flag_long() {
    let tmp = TempDir::new().unwrap();
    create_test_project(tmp.path());

    let config_path = tmp.path().join("my-anodize.yaml");
    fs::write(&config_path, r#"
project_name: test-project
crates:
  - name: test-project
    path: "."
    tag_template: "v{{ .Version }}"
"#).unwrap();

    // Use --config (long form) to point to the custom config
    let output = Command::new(env!("CARGO_BIN_EXE_anodize"))
        .args(["--config", config_path.to_str().unwrap(), "check"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "check --config should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_check_with_config_flag_nonexistent() {
    let output = Command::new(env!("CARGO_BIN_EXE_anodize"))
        .args(["-f", "/tmp/does-not-exist-anodize.yaml", "check"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("config file not found"),
        "expected 'config file not found' error, got: {}",
        stderr
    );
}

#[test]
fn test_release_help_shows_timeout_flag() {
    let output = Command::new(env!("CARGO_BIN_EXE_anodize"))
        .args(["release", "--help"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--timeout"),
        "release --help should show --timeout flag, got: {}",
        stdout
    );
}

#[test]
fn test_build_help_shows_timeout_flag() {
    let output = Command::new(env!("CARGO_BIN_EXE_anodize"))
        .args(["build", "--help"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--timeout"),
        "build --help should show --timeout flag, got: {}",
        stdout
    );
}

#[test]
fn test_timeout_kills_long_running_release() {
    let tmp = TempDir::new().unwrap();
    create_test_project(tmp.path());
    init_git_repo(tmp.path());

    // Config with a before-hook that sleeps for 60 seconds (much longer than our timeout)
    create_config(tmp.path(), r#"
project_name: test-project
before:
  hooks:
    - "sleep 60"
crates:
  - name: test-project
    path: "."
    tag_template: "v{{ .Version }}"
"#);

    let start = Instant::now();

    // Use spawn + try_wait instead of output(). When std::process::exit(124)
    // fires from the watchdog thread, the grandchild `sleep 60` may still
    // hold inherited pipe fds open, causing output() to block until that
    // process also exits. By discarding stdout/stderr with Stdio::null()
    // and polling try_wait(), we detect the exit immediately.
    let mut child = Command::new(env!("CARGO_BIN_EXE_anodize"))
        .args(["release", "--timeout", "1s"])
        .current_dir(tmp.path())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();

    // Poll for completion with a generous timeout
    let poll_deadline = Instant::now() + Duration::from_secs(10);
    let exit_status = loop {
        match child.try_wait().unwrap() {
            Some(status) => break status,
            None => {
                if Instant::now() > poll_deadline {
                    child.kill().ok();
                    panic!("anodize process did not exit within 10s (timeout was 1s)");
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    };
    let elapsed = start.elapsed();

    // Should have been killed by timeout
    assert!(
        !exit_status.success(),
        "release with 1s timeout on a 60s sleep should fail"
    );

    // Verify exit code 124 (conventional timeout exit code)
    assert_eq!(
        exit_status.code(),
        Some(124),
        "expected exit code 124 for timeout, got {:?}",
        exit_status.code()
    );

    // The process should finish in well under 10s (timeout is 1s)
    assert!(
        elapsed < Duration::from_secs(10),
        "process should have been killed by timeout quickly, but took {:?}",
        elapsed
    );
}

#[test]
fn test_completion_bash_produces_output() {
    let output = Command::new(env!("CARGO_BIN_EXE_anodize"))
        .args(["completion", "bash"])
        .output()
        .unwrap();

    assert!(output.status.success(), "completion bash should succeed: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty(), "bash completions should not be empty");
    assert!(stdout.contains("anodize"), "bash completions should reference 'anodize'");
}

#[test]
fn test_completion_zsh_produces_output() {
    let output = Command::new(env!("CARGO_BIN_EXE_anodize"))
        .args(["completion", "zsh"])
        .output()
        .unwrap();

    assert!(output.status.success(), "completion zsh should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty(), "zsh completions should not be empty");
}

#[test]
fn test_healthcheck_succeeds() {
    let output = Command::new(env!("CARGO_BIN_EXE_anodize"))
        .arg("healthcheck")
        .output()
        .unwrap();

    assert!(output.status.success(), "healthcheck should succeed: {}", String::from_utf8_lossy(&output.stderr));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Health Check"), "healthcheck should print header");
    assert!(stderr.contains("cargo"), "healthcheck should check cargo");
}

#[test]
fn test_release_help_shows_new_flags() {
    let output = Command::new(env!("CARGO_BIN_EXE_anodize"))
        .args(["release", "--help"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--parallelism"), "release --help should show --parallelism: {}", stdout);
    assert!(stdout.contains("--auto-snapshot"), "release --help should show --auto-snapshot: {}", stdout);
    assert!(stdout.contains("--single-target"), "release --help should show --single-target: {}", stdout);
    assert!(stdout.contains("--release-notes"), "release --help should show --release-notes: {}", stdout);
}

#[test]
fn test_build_help_shows_new_flags() {
    let output = Command::new(env!("CARGO_BIN_EXE_anodize"))
        .args(["build", "--help"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--parallelism"), "build --help should show --parallelism: {}", stdout);
    assert!(stdout.contains("--single-target"), "build --help should show --single-target: {}", stdout);
}

#[test]
fn test_release_invalid_timeout_value() {
    let tmp = TempDir::new().unwrap();
    create_test_project(tmp.path());
    create_config(tmp.path(), r#"
project_name: test-project
crates:
  - name: test-project
    path: "."
    tag_template: "v{{ .Version }}"
"#);

    let output = Command::new(env!("CARGO_BIN_EXE_anodize"))
        .args(["release", "--timeout", "notavalidtimeout"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("invalid --timeout value"),
        "stderr should report invalid timeout, got: {}",
        stderr
    );
}
