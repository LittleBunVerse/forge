use std::path::{Component, Path, PathBuf};

use anyhow::{Result, anyhow};

pub fn expand_tilde(path: &str) -> Result<String> {
    if path.is_empty() {
        return Ok(String::new());
    }

    if path == "~" {
        return Ok(user_home_dir()?.to_string_lossy().to_string());
    }

    if path.starts_with("~/") || path.starts_with("~\\") {
        return Ok(user_home_dir()?
            .join(&path[2..])
            .to_string_lossy()
            .to_string());
    }

    Ok(path.to_string())
}

pub fn normalize_root(root: &str) -> Result<String> {
    let expanded = expand_tilde(root)?;
    let absolute = absolute_clean(Path::new(&expanded))?;

    let metadata = std::fs::metadata(&absolute).map_err(|err| anyhow!("根目录不存在：{err}"))?;
    if !metadata.is_dir() {
        return Err(anyhow!("根路径不是目录：{}", absolute.to_string_lossy()));
    }

    Ok(absolute.to_string_lossy().to_string())
}

pub fn detect_default_root() -> String {
    let Ok(home) = user_home_dir() else {
        return ".".to_string();
    };

    for candidate in [home.join("Projects"), home.join("IdeaProjects")] {
        if candidate.is_dir() {
            return candidate.to_string_lossy().to_string();
        }
    }

    ".".to_string()
}

pub(crate) fn absolute_clean(path: &Path) -> Result<PathBuf> {
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|err| anyhow!("解析绝对路径失败：{err}"))?
            .join(path)
    };
    Ok(normalize_components(&joined))
}

fn normalize_components(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(part) => normalized.push(part),
            Component::RootDir | Component::Prefix(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }

    if normalized.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        normalized
    }
}

fn user_home_dir() -> Result<PathBuf> {
    if let Some(value) = std::env::var_os("HOME").filter(|value| !value.is_empty()) {
        return Ok(PathBuf::from(value));
    }
    if let Some(value) = std::env::var_os("USERPROFILE").filter(|value| !value.is_empty()) {
        return Ok(PathBuf::from(value));
    }

    match (
        std::env::var_os("HOMEDRIVE").filter(|value| !value.is_empty()),
        std::env::var_os("HOMEPATH").filter(|value| !value.is_empty()),
    ) {
        (Some(drive), Some(path)) => Ok(PathBuf::from(format!(
            "{}{}",
            PathBuf::from(drive).to_string_lossy(),
            PathBuf::from(path).to_string_lossy()
        ))),
        _ => Err(anyhow!("获取用户 HOME 目录失败")),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{absolute_clean, detect_default_root, expand_tilde, normalize_root};

    #[test]
    fn test_expand_tilde() {
        let home = match std::env::var("HOME") {
            Ok(value) => value,
            Err(err) => {
                eprintln!("无法获取 HOME 目录：{err}");
                return;
            }
        };

        let got = expand_tilde("~").expect("ExpandTilde(~) 返回错误");
        assert_eq!(got, home, "ExpandTilde(~) 不符合预期");

        let got = expand_tilde("~/abc").expect("ExpandTilde(~/abc) 返回错误");
        assert_eq!(
            got,
            PathBuf::from(&home).join("abc").to_string_lossy(),
            "ExpandTilde(~/abc) 不符合预期",
        );

        let got = expand_tilde("~\\abc").expect("ExpandTilde(~\\\\abc) 返回错误");
        assert_eq!(
            got,
            PathBuf::from(&home).join("abc").to_string_lossy(),
            "ExpandTilde(~\\\\abc) 不符合预期",
        );
    }

    #[test]
    fn test_normalize_root() {
        let tmp = tempfile::tempdir().expect("创建临时目录失败");

        let got =
            normalize_root(tmp.path().to_string_lossy().as_ref()).expect("NormalizeRoot 返回错误");
        let want = absolute_clean(tmp.path())
            .expect("解析绝对路径失败")
            .to_string_lossy()
            .to_string();

        assert_eq!(got, want, "NormalizeRoot 不符合预期");
    }

    #[test]
    fn test_detect_default_root_falls_back_to_current_dir() {
        let current = std::env::current_dir().expect("读取当前目录失败");
        let old_home = std::env::var_os("HOME");
        let fake_home = tempfile::tempdir().expect("创建临时 HOME 失败");
        unsafe {
            std::env::set_var("HOME", fake_home.path());
        }

        let got = detect_default_root();

        match old_home {
            Some(value) => unsafe {
                std::env::set_var("HOME", value);
            },
            None => unsafe {
                std::env::remove_var("HOME");
            },
        }

        assert_eq!(
            got, ".",
            "没有 Projects/IdeaProjects 时应回退到当前目录标记"
        );
        assert!(current.exists(), "当前目录应存在");
    }
}
