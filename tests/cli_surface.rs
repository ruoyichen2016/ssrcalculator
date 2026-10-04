//! Command-surface regression test.
//!
//! The CLI is assembled from three places that must agree: the help text lists
//! commands, `main.rs` routes them to a handler module, and each handler module
//! matches on the command name.  Nothing in the compiler checks that those three
//! agree, and the failure mode is silent: `--help` keeps advertising a command
//! that the binary answers with "Unknown command".  Splitting `main.rs` into
//! `src/cli/` modules multiplies the places that can drift.
//!
//! This test therefore drives the real binary: every command advertised in
//! `--help` must not answer with "Unknown command".

use std::process::{Command, Stdio};

/// Commands that legitimately do not terminate on their own.
///
/// `web` starts a blocking HTTP server.  (`interact` is handled by pointing its
/// stdin at /dev/null, which ends the session immediately.)
const BLOCKING: &[&str] = &["web"];

fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_springsharp")
}

/// Parse the command names out of the `COMMANDS:` section of `--help`.
fn advertised_commands() -> Vec<String> {
    let out = Command::new(binary())
        .arg("help")
        .output()
        .expect("failed to run `springsharp help`");
    let text = String::from_utf8_lossy(&out.stdout);

    let mut commands = Vec::new();
    let mut in_commands = false;
    for line in text.lines() {
        // The trailing "Grouped aliases" block documents aliases, not commands.
        if line.starts_with("Grouped aliases") {
            break;
        }
        if line.starts_with("COMMANDS:") {
            in_commands = true;
            continue;
        }
        if !in_commands {
            continue;
        }
        let Some(rest) = line.strip_prefix("  ") else {
            continue;
        };
        let name = rest.split_whitespace().next().unwrap_or("");
        if !name.is_empty() && name.starts_with(|c: char| c.is_ascii_lowercase()) {
            commands.push(name.to_string());
        }
    }
    commands
}

#[test]
fn help_text_lists_the_command_surface() {
    let commands = advertised_commands();
    assert!(
        commands.len() > 40,
        "expected the help text to advertise many commands, parsed {}: {commands:?}",
        commands.len()
    );
}

#[test]
fn every_advertised_command_is_routed() {
    let commands = advertised_commands();
    let mut unrouted = Vec::new();

    for cmd in &commands {
        if BLOCKING.contains(&cmd.as_str()) {
            continue;
        }
        // No arguments on purpose: a routed command answers with a usage error
        // (or does its default thing); an unrouted one answers "Unknown command".
        let out = Command::new(binary())
            .arg(cmd)
            .stdin(Stdio::null())
            .output()
            .unwrap_or_else(|e| panic!("failed to run `springsharp {cmd}`: {e}"));

        let combined = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        if combined.contains("Unknown command") {
            unrouted.push(cmd.clone());
        }
    }

    assert!(
        unrouted.is_empty(),
        "these commands are advertised in --help but not routed in main.rs: {unrouted:?}"
    );
}
