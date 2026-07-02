pub(crate) fn find_executable(cmd: &str) -> Option<std::path::PathBuf> {
    match which::which(cmd) {
        Ok(path) => {
            if (is_executable(&path)) {
                return Some(path);
            }
            return None;
        }
        Err(_) => {
            return None;
        }
    }
}

fn is_executable(path: &std::path::PathBuf) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        return path
            .metadata()
            .map(|m| m.is_file() && (m.permissions().mode() & 0o111 != 0))
            .unwrap_or(false);
    }
    #[cfg(windows)]
    {
        let exts = std::env::var_os("PATHEXT")
            .map(|val| std::env::split_paths(&val).collect::<Vec<_>>())
            .unwrap_or_else(|| {
                vec![
                    std::path::PathBuf::from(".EXE"),
                    std::path::PathBuf::from(".CMD"),
                    std::path::PathBuf::from(".BAT"),
                ]
            });
        return path.extension().map_or(false, |ext| {
            let ext_upper = format!(".{}", ext.to_string_lossy().to_ascii_uppercase());
            return exts.iter().any(|ele| {
                let ele_str = ele.to_string_lossy().to_ascii_uppercase();
                ele_str == ext_upper
            });
        });
    }
}
