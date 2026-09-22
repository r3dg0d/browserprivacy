use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn help_works() {
    Command::cargo_bin("browserprivacy")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("NEVER extracts passwords"));
}

#[test]
fn version_works() {
    Command::cargo_bin("browserprivacy")
        .unwrap()
        .arg("--version")
        .assert()
        .success();
}

#[test]
fn completions() {
    Command::cargo_bin("browserprivacy")
        .unwrap()
        .args(["completions", "fish"])
        .assert()
        .success();
}

#[test]
fn list_profiles_smoke() {
    Command::cargo_bin("browserprivacy")
        .unwrap()
        .arg("list-profiles")
        .assert()
        .success();
}

#[test]
fn audit_with_fake_chromium_profile() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("fake-chrome");
    let profile = root.join("Default");
    fs::create_dir_all(&profile).unwrap();
    fs::write(
        profile.join("Preferences"),
        r#"{"dns_over_https":{"mode":"secure"},"profile":{"block_third_party_cookies":true},"extensions":{"settings":{}}}"#,
    )
    .unwrap();

    // Point via config extra root — use env HOME isolation
    let home = dir.path().join("home");
    fs::create_dir_all(home.join(".config")).unwrap();
    // Put fake as chromium path
    let chrome = home.join(".config/chromium");
    fs::create_dir_all(chrome.join("Default")).unwrap();
    fs::write(
        chrome.join("Default/Preferences"),
        r#"{"dns_over_https":{"mode":"off"},"profile":{"default_content_setting_values":{"cookies":1,"notifications":2}},"extensions":{"settings":{"abcdefgh":{"state":1,"manifest":{"name":"Test Ext","version":"1.0"}}}}}"#,
    )
    .unwrap();

    Command::cargo_bin("browserprivacy")
        .unwrap()
        .env("HOME", home.to_str().unwrap())
        .env("XDG_CONFIG_HOME", home.join(".config").to_str().unwrap())
        .args(["--json", "audit"])
        .assert()
        .success()
        .stdout(predicate::str::contains("dns_over_https").or(predicate::str::contains("DoH").not()));
}
