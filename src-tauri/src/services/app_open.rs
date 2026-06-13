use std::process::Command;

const ALLOWED_EXTERNAL_URLS: &[&str] = &[
    "https://github.com/xiaofi/agent-island",
    "https://github.com/xiaofi/agent-island/releases",
];

pub fn open_path(path: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let mut command = {
        let mut command = Command::new("open");
        command.arg(path);
        command
    };

    #[cfg(target_os = "windows")]
    let mut command = {
        let mut command = Command::new("explorer");
        command.arg(path);
        command
    };

    #[cfg(all(unix, not(target_os = "macos")))]
    let mut command = {
        let mut command = Command::new("xdg-open");
        command.arg(path);
        command
    };

    command.spawn().map_err(|error| error.to_string())?;
    Ok(())
}

pub fn open_url(url: &str) -> Result<(), String> {
    if !ALLOWED_EXTERNAL_URLS.contains(&url) {
        return Err(format!("unsupported external URL: {url}"));
    }

    open_path(url)
}
