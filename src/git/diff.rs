use std::process::Command;

pub fn get_diff(file_path: &str, width: u16) -> Vec<u8> {
    // Try delta first
    if let Ok(output) = try_delta(file_path, width) {
        return output;
    }

    // Fallback to git diff
    try_git_diff(file_path).unwrap_or_else(|_| b"Failed to get diff".to_vec())
}

fn try_delta(file_path: &str, width: u16) -> Result<Vec<u8>, ()> {
    // Check if delta is available
    if which::which("delta").is_err() {
        return Err(());
    }

    // Get git diff first
    let git_output = Command::new("git")
        .args(["diff", file_path])
        .output()
        .map_err(|_| ())?;

    let diff_input = if git_output.stdout.is_empty() {
        // Try for untracked/new files
        let output = Command::new("git")
            .args(["diff", "--no-index", "/dev/null", file_path])
            .output()
            .map_err(|_| ())?;
        output.stdout
    } else {
        git_output.stdout
    };

    if diff_input.is_empty() {
        return Err(());
    }

    // Pipe through delta with terminal width
    let mut delta_process = Command::new("delta")
        .args(["--width", &width.to_string()])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|_| ())?;

    use std::io::Write;
    if let Some(ref mut stdin) = delta_process.stdin {
        let _ = stdin.write_all(&diff_input);
    }
    // Close stdin by dropping it
    delta_process.stdin.take();

    let output = delta_process.wait_with_output().map_err(|_| ())?;
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
