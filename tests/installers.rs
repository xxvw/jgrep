//! Offline smoke coverage for the versioned release installers.
//!
//! These tests synthesize the same target-qualified archive and SHA-256
//! manifest shape produced by the release workflow. They never contact
//! GitHub, so the normal native `cargo xtask ci` contract exercises the
//! installer paths before a GitHub Release exists.

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

#[cfg(windows)]
use std::ffi::OsString;

use sha2::{Digest, Sha256};

const VERSION: &str = "v0.1.0";

fn project_path(relative: impl AsRef<Path>) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn jgrep_binary() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_jgrep")
        .map(PathBuf::from)
        .expect("Cargo should expose the jgrep binary to integration tests")
}

fn sha256(path: &Path) -> String {
    let bytes = fs::read(path).expect("read synthesized archive");
    format!("{:x}", Sha256::digest(bytes))
}

fn write_checksum(asset_directory: &Path, archive_name: &str) {
    let archive = asset_directory.join(archive_name);
    fs::write(
        asset_directory.join(format!("{archive_name}.sha256")),
        format!("{}  {archive_name}\n", sha256(&archive)),
    )
    .expect("write synthesized archive checksum");
}

fn replace_checksum_with_manifest(asset_directory: &Path, archive_name: &str) {
    let archive = asset_directory.join(archive_name);
    fs::remove_file(asset_directory.join(format!("{archive_name}.sha256")))
        .expect("remove per-archive checksum before manifest fallback smoke test");
    fs::write(
        asset_directory.join("SHA256SUMS"),
        format!(
            "{}  unrelated-release-asset\n{}  {archive_name}\n",
            sha256(&archive),
            sha256(&archive),
        ),
    )
    .expect("write synthesized multi-asset checksum manifest");
}

fn assert_success(output: &Output, action: &str) {
    assert!(
        output.status.success(),
        "{action} failed with {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

fn assert_installed(binary: &Path) {
    assert!(
        binary.is_file(),
        "installer did not create {}",
        binary.display()
    );
    let output = Command::new(binary)
        .arg("--version")
        .output()
        .expect("run installed jgrep");
    assert_success(&output, "installed jgrep --version");
    assert!(
        String::from_utf8_lossy(&output.stdout).starts_with("jgrep 0.1.0"),
        "installed executable reported an unexpected version: {output:?}"
    );
}

#[cfg(unix)]
fn unix_target() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        host => panic!("no published Unix release target for {host:?}"),
    }
}

#[cfg(unix)]
fn run_unix_installer(
    asset_directory: &Path,
    target: &str,
    install_directory: &Path,
    force: bool,
) -> Output {
    let mut command = Command::new("bash");
    command
        .arg(project_path("scripts/install.sh"))
        .args(["--asset-dir"])
        .arg(asset_directory)
        .args(["--version", VERSION, "--target", target, "--install-dir"])
        .arg(install_directory);
    if force {
        command.arg("--force");
    }
    command.output().expect("run Unix installer")
}

#[cfg(unix)]
#[test]
fn unix_installer_verifies_and_installs_a_local_release_archive() {
    let temp = tempfile::tempdir().expect("temporary installer fixture");
    let target = unix_target();
    let asset_directory = temp.path().join("assets");
    let package_root = format!("localjev-grep-{VERSION}-{target}");
    let stage = asset_directory.join(&package_root);
    fs::create_dir_all(&stage).expect("create release staging directory");
    fs::copy(jgrep_binary(), stage.join("jgrep"))
        .expect("copy jgrep into release staging directory");

    let archive_name = format!("{package_root}.tar.gz");
    let archive_path = asset_directory.join(&archive_name);
    let output = Command::new("tar")
        .arg("-czf")
        .arg(&archive_path)
        .arg("-C")
        .arg(&asset_directory)
        .arg(&package_root)
        .output()
        .expect("create synthesized tar archive");
    assert_success(&output, "create synthesized Unix release archive");
    write_checksum(&asset_directory, &archive_name);

    let install_directory = temp.path().join("bin");
    let first = run_unix_installer(&asset_directory, target, &install_directory, false);
    assert_success(&first, "first Unix installer invocation");
    let installed = install_directory.join("jgrep");
    assert_installed(&installed);

    let overwrite = run_unix_installer(&asset_directory, target, &install_directory, false);
    assert!(
        !overwrite.status.success(),
        "installer must not overwrite an executable without --force"
    );
    assert!(
        String::from_utf8_lossy(&overwrite.stderr).contains("--force"),
        "unexpected overwrite diagnostic: {overwrite:?}"
    );

    replace_checksum_with_manifest(&asset_directory, &archive_name);
    let forced = run_unix_installer(&asset_directory, target, &install_directory, true);
    assert_success(&forced, "forced Unix installer invocation");
    assert_installed(&installed);

    let symlink_install_directory = temp.path().join("symlink-bin");
    let symlink_target_directory = temp.path().join("symlink-target");
    fs::create_dir_all(&symlink_install_directory).expect("create symlink installation directory");
    fs::create_dir_all(&symlink_target_directory).expect("create symlink target directory");
    std::os::unix::fs::symlink(
        &symlink_target_directory,
        symlink_install_directory.join("jgrep"),
    )
    .expect("create destination symlink");
    let symlink_attempt =
        run_unix_installer(&asset_directory, target, &symlink_install_directory, true);
    assert!(
        !symlink_attempt.status.success(),
        "installer must reject a symbolic-link destination even with --force"
    );
    assert!(
        String::from_utf8_lossy(&symlink_attempt.stderr).contains("symbolic link"),
        "unexpected symbolic-link diagnostic: {symlink_attempt:?}"
    );
    assert!(
        fs::read_dir(&symlink_target_directory)
            .expect("read protected symlink target")
            .next()
            .is_none(),
        "installer must not place a staged file inside a symlink target"
    );

    let mismatched_asset_directory = temp.path().join("mismatched-assets");
    let mismatched_stage = mismatched_asset_directory.join(&package_root);
    fs::create_dir_all(&mismatched_stage).expect("create mismatched release staging directory");
    fs::write(
        mismatched_stage.join("jgrep"),
        b"#!/bin/sh\nprintf 'jgrep 9.9.9\\n'\n",
    )
    .expect("write mismatched jgrep fixture");
    let mismatched_archive = mismatched_asset_directory.join(&archive_name);
    let archive_output = Command::new("tar")
        .arg("-czf")
        .arg(&mismatched_archive)
        .arg("-C")
        .arg(&mismatched_asset_directory)
        .arg(&package_root)
        .output()
        .expect("create mismatched Unix release archive");
    assert_success(&archive_output, "create mismatched Unix release archive");
    write_checksum(&mismatched_asset_directory, &archive_name);
    let mismatched_install_directory = temp.path().join("mismatched-bin");
    let mismatched_attempt = run_unix_installer(
        &mismatched_asset_directory,
        target,
        &mismatched_install_directory,
        false,
    );
    assert!(
        !mismatched_attempt.status.success(),
        "installer must reject an archive whose executable reports another version"
    );
    assert!(
        String::from_utf8_lossy(&mismatched_attempt.stderr).contains("version mismatch"),
        "unexpected version-mismatch diagnostic: {mismatched_attempt:?}"
    );
    assert!(
        !mismatched_install_directory.join("jgrep").exists(),
        "installer must not leave a mismatched executable behind"
    );
}

#[cfg(unix)]
#[test]
fn installer_and_agent_template_static_contract_is_valid() {
    let output = Command::new("bash")
        .arg(project_path("scripts/validate-installers.sh"))
        .output()
        .expect("run installer/template static validator");
    assert_success(&output, "installer/template static validation");
}

#[cfg(windows)]
fn run_powershell(arguments: impl IntoIterator<Item = OsString>) -> Output {
    Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
        ])
        .args(arguments)
        .output()
        .expect("run PowerShell")
}

#[cfg(windows)]
fn run_windows_installer(
    asset_directory: &Path,
    install_directory: &Path,
    version: &str,
    force: bool,
) -> Output {
    let mut arguments = vec![
        OsString::from("-File"),
        project_path("scripts/install.ps1").into_os_string(),
    ];
    arguments.extend([
        OsString::from("-AssetDirectory"),
        asset_directory.as_os_str().to_owned(),
        OsString::from("-Version"),
        OsString::from(version),
        OsString::from("-Target"),
        OsString::from("x86_64-pc-windows-msvc"),
        OsString::from("-InstallDir"),
        install_directory.as_os_str().to_owned(),
    ]);
    if force {
        arguments.push(OsString::from("-Force"));
    }
    run_powershell(arguments)
}

#[cfg(windows)]
#[test]
fn windows_installer_verifies_and_installs_a_local_release_archive() {
    let temp = tempfile::tempdir().expect("temporary installer fixture");
    let target = "x86_64-pc-windows-msvc";
    let asset_directory = temp.path().join("assets");
    let package_root = format!("localjev-grep-{VERSION}-{target}");
    let stage = asset_directory.join(&package_root);
    fs::create_dir_all(&stage).expect("create release staging directory");
    fs::copy(jgrep_binary(), stage.join("jgrep.exe"))
        .expect("copy jgrep into release staging directory");

    let archive_name = format!("{package_root}.zip");
    let archive_path = asset_directory.join(&archive_name);
    let archive_output = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            "Compress-Archive -LiteralPath $env:JGREP_INSTALLER_STAGE -DestinationPath $env:JGREP_INSTALLER_ARCHIVE -Force",
        ])
        .env("JGREP_INSTALLER_STAGE", &stage)
        .env("JGREP_INSTALLER_ARCHIVE", &archive_path)
        .output()
        .expect("create synthesized zip archive");
    assert_success(
        &archive_output,
        "create synthesized Windows release archive",
    );
    write_checksum(&asset_directory, &archive_name);

    let install_directory = temp.path().join("bin");
    let first = run_windows_installer(&asset_directory, &install_directory, VERSION, false);
    assert_success(&first, "first Windows installer invocation");
    let installed = install_directory.join("jgrep.exe");
    assert_installed(&installed);

    let overwrite = run_windows_installer(&asset_directory, &install_directory, VERSION, false);
    assert!(
        !overwrite.status.success(),
        "installer must not overwrite an executable without -Force"
    );
    assert!(
        String::from_utf8_lossy(&overwrite.stderr).contains("-Force"),
        "unexpected overwrite diagnostic: {overwrite:?}"
    );

    replace_checksum_with_manifest(&asset_directory, &archive_name);
    let forced = run_windows_installer(&asset_directory, &install_directory, VERSION, true);
    assert_success(&forced, "forced Windows installer invocation");
    assert_installed(&installed);

    let mismatched_version = "v9.9.9";
    let mismatched_asset_directory = temp.path().join("mismatched-assets");
    let mismatched_package_root = format!("localjev-grep-{mismatched_version}-{target}");
    let mismatched_stage = mismatched_asset_directory.join(&mismatched_package_root);
    fs::create_dir_all(&mismatched_stage).expect("create mismatched release staging directory");
    fs::copy(jgrep_binary(), mismatched_stage.join("jgrep.exe"))
        .expect("copy jgrep into mismatched release staging directory");
    let mismatched_archive_name = format!("{mismatched_package_root}.zip");
    let mismatched_archive_path = mismatched_asset_directory.join(&mismatched_archive_name);
    let mismatched_archive_output = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            "Compress-Archive -LiteralPath $env:JGREP_INSTALLER_STAGE -DestinationPath $env:JGREP_INSTALLER_ARCHIVE -Force",
        ])
        .env("JGREP_INSTALLER_STAGE", &mismatched_stage)
        .env("JGREP_INSTALLER_ARCHIVE", &mismatched_archive_path)
        .output()
        .expect("create mismatched Windows release archive");
    assert_success(
        &mismatched_archive_output,
        "create mismatched Windows release archive",
    );
    write_checksum(&mismatched_asset_directory, &mismatched_archive_name);
    let mismatched_install_directory = temp.path().join("mismatched-bin");
    let mismatched_attempt = run_windows_installer(
        &mismatched_asset_directory,
        &mismatched_install_directory,
        mismatched_version,
        false,
    );
    assert!(
        !mismatched_attempt.status.success(),
        "installer must reject an archive whose executable reports another version"
    );
    assert!(
        String::from_utf8_lossy(&mismatched_attempt.stderr).contains("version mismatch"),
        "unexpected version-mismatch diagnostic: {mismatched_attempt:?}"
    );
    assert!(
        !mismatched_install_directory.join("jgrep.exe").exists(),
        "installer must not leave a mismatched executable behind"
    );

    let reparse_install_directory = temp.path().join("reparse-bin");
    fs::create_dir_all(&reparse_install_directory).expect("create reparse installation directory");
    let reparse_target_directory = temp.path().join("reparse-target");
    fs::create_dir_all(&reparse_target_directory).expect("create reparse target directory");
    let junction_output = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            "New-Item -ItemType Junction -Path $env:JGREP_INSTALLER_LINK -Target $env:JGREP_INSTALLER_LINK_TARGET | Out-Null",
        ])
        .env(
            "JGREP_INSTALLER_LINK",
            reparse_install_directory.join("jgrep.exe"),
        )
        .env("JGREP_INSTALLER_LINK_TARGET", &reparse_target_directory)
        .output()
        .expect("create destination junction");
    assert_success(&junction_output, "create destination junction");
    let reparse_attempt =
        run_windows_installer(&asset_directory, &reparse_install_directory, VERSION, true);
    assert!(
        !reparse_attempt.status.success(),
        "installer must reject a reparse-point destination even with -Force"
    );
    assert!(
        String::from_utf8_lossy(&reparse_attempt.stderr).contains("reparse point"),
        "unexpected reparse-point diagnostic: {reparse_attempt:?}"
    );
    assert!(
        fs::read_dir(&reparse_target_directory)
            .expect("read protected reparse target")
            .next()
            .is_none(),
        "installer must not place a staged file inside a reparse target"
    );

    let ancestor_target_directory = temp.path().join("ancestor-reparse-target");
    fs::create_dir_all(&ancestor_target_directory)
        .expect("create ancestor reparse target directory");
    let ancestor_link = temp.path().join("ancestor-reparse-link");
    let ancestor_junction_output = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            "New-Item -ItemType Junction -Path $env:JGREP_INSTALLER_LINK -Target $env:JGREP_INSTALLER_LINK_TARGET | Out-Null",
        ])
        .env("JGREP_INSTALLER_LINK", &ancestor_link)
        .env(
            "JGREP_INSTALLER_LINK_TARGET",
            &ancestor_target_directory,
        )
        .output()
        .expect("create ancestor junction");
    assert_success(&ancestor_junction_output, "create ancestor junction");
    let ancestor_attempt =
        run_windows_installer(&asset_directory, &ancestor_link.join("bin"), VERSION, false);
    assert!(
        !ancestor_attempt.status.success(),
        "installer must reject an ancestor reparse point"
    );
    assert!(
        String::from_utf8_lossy(&ancestor_attempt.stderr).contains("reparse point"),
        "unexpected ancestor reparse-point diagnostic: {ancestor_attempt:?}"
    );
    assert!(
        fs::read_dir(&ancestor_target_directory)
            .expect("read protected ancestor reparse target")
            .next()
            .is_none(),
        "installer must not create a directory or place a file through an ancestor reparse point"
    );
}
