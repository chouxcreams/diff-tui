use std::io::Write;
use std::process::{Command, Stdio};

use crate::config::DiffConfig;

pub fn get_diff(file_path: &str, width: u16, config: &DiffConfig) -> Vec<u8> {
    match config.tool.as_str() {
        "auto" => {
            // Try delta first, then fallback to git diff
            if let Ok(output) = try_tool("delta", file_path, width, &["--width"]) {
                return output;
            }
            try_git_diff(file_path).unwrap_or_else(|_| b"Failed to get diff".to_vec())
        }
        "git" => try_git_diff(file_path).unwrap_or_else(|_| b"Failed to get diff".to_vec()),
        tool => {
            // Try the specified tool
            if let Ok(output) = try_tool(tool, file_path, width, &config.args) {
                return output;
            }
            // Fallback to git diff
            try_git_diff(file_path).unwrap_or_else(|_| b"Failed to get diff".to_vec())
        }
    }
}

fn try_tool(tool_name: &str, file_path: &str, width: u16, extra_args: &[impl AsRef<str>]) -> Result<Vec<u8>, ()> {
    // Check if the tool is available
    if which::which(tool_name).is_err() {
        return Err(());
    }

    // Get git diff first
    let diff_input = get_git_diff_output(file_path)?;

    if diff_input.is_empty() {
        return Err(());
    }

    // Build command with arguments
    let mut cmd = Command::new(tool_name);

    // Add width argument for delta
    if tool_name == "delta" {
        cmd.args(["--width", &width.to_string()]);
    }

    // Add extra arguments from config
    for arg in extra_args {
        let arg_str = arg.as_ref();
        // Skip --width for delta as we already added it
        if tool_name == "delta" && arg_str == "--width" {
            continue;
        }
        cmd.arg(arg_str);
    }

    // Pipe diff through the tool
    let mut process = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|_| ())?;

    if let Some(ref mut stdin) = process.stdin {
        let _ = stdin.write_all(&diff_input);
    }
    process.stdin.take();

    let output = process.wait_with_output().map_err(|_| ())?;
    Ok(output.stdout)
}

fn get_git_diff_output(file_path: &str) -> Result<Vec<u8>, ()> {
    let output = Command::new("git")
        .args(["diff", file_path])
        .output()
        .map_err(|_| ())?;

    if !output.stdout.is_empty() {
        return Ok(output.stdout);
    }

    // Try for untracked/new files
    let output = Command::new("git")
        .args(["diff", "--no-index", "/dev/null", file_path])
        .output()
        .map_err(|_| ())?;

    Ok(output.stdout)
}

fn try_git_diff(file_path: &str) -> Result<Vec<u8>, ()> {
    let output = Command::new("git")
        .args(["diff", "--color=always", file_path])
        .output()
        .map_err(|_| ())?;

    if !output.stdout.is_empty() {
        return Ok(output.stdout);
    }

    // Try for untracked/new files
    let output = Command::new("git")
        .args(["diff", "--color=always", "--no-index", "/dev/null", file_path])
        .output()
        .map_err(|_| ())?;

    Ok(output.stdout)
}
