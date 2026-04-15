use anyhow::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dir {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct Options {
    pub ignore_names: Option<std::collections::HashSet<String>>,
}

pub fn default_ignore_names() -> std::collections::HashSet<String> {
    ["node_modules", "target", "dist", "vendor"]
        .into_iter()
        .map(str::to_string)
        .collect()
}

pub fn list_dirs(_root: &str, _options: Options) -> Result<Vec<Dir>> {
    let root = _root.trim();
    if root.is_empty() {
        return Err(anyhow::anyhow!("root 不能为空"));
    }

    let entries = std::fs::read_dir(root).map_err(|err| anyhow::anyhow!("读取目录失败：{err}"))?;

    let ignore = _options.ignore_names.unwrap_or_else(default_ignore_names);
    let mut dirs = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|err| anyhow::anyhow!("读取目录失败：{err}"))?;
        let name = entry.file_name().to_string_lossy().to_string();

        if name.starts_with('.') || ignore.contains(&name) {
            continue;
        }

        let file_type = entry
            .file_type()
            .map_err(|err| anyhow::anyhow!("读取目录失败：{err}"))?;

        let is_dir = if file_type.is_dir() {
            true
        } else if file_type.is_symlink() {
            entry.metadata().map(|meta| meta.is_dir()).unwrap_or(false)
        } else {
            false
        };

        if !is_dir {
            continue;
        }

        dirs.push(Dir {
            name: name.clone(),
            path: entry.path().to_string_lossy().to_string(),
        });
    }

    dirs.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(dirs)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{Options, default_ignore_names, list_dirs};

    #[test]
    fn test_list_dirs_filter_and_sort() {
        let root = tempfile::tempdir().expect("创建临时目录失败");

        fs::create_dir_all(root.path().join("b")).expect("创建 b 目录失败");
        fs::create_dir_all(root.path().join("a")).expect("创建 a 目录失败");
        fs::create_dir_all(root.path().join(".git")).expect("创建 .git 目录失败");
        fs::create_dir_all(root.path().join("node_modules")).expect("创建 node_modules 目录失败");
        fs::write(root.path().join("file.txt"), b"x").expect("写入文件失败");

        let got = list_dirs(
            root.path().to_string_lossy().as_ref(),
            Options {
                ignore_names: Some(default_ignore_names()),
            },
        )
        .expect("ListDirs 返回错误");

        assert_eq!(got.len(), 2, "目录数量不符合预期");
        assert_eq!(got[0].name, "a", "排序或过滤不正确");
        assert_eq!(got[1].name, "b", "排序或过滤不正确");
    }

    #[test]
    fn test_list_dirs_empty_root() {
        let err = list_dirs(
            "",
            Options {
                ignore_names: Some(default_ignore_names()),
            },
        )
        .expect_err("root 为空时应报错");

        assert!(err.to_string().contains("root 不能为空"));
    }
}
