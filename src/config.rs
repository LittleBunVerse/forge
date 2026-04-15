use std::io::Write;
use std::path::PathBuf;

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

use crate::pathutil::{absolute_clean, expand_tilde};

pub const CURRENT_APP_NAME: &str = "forge";
pub const LEGACY_APP_NAME: &str = "aidev";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandConfig {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub root: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub commands: Vec<CommandConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub projects: Vec<ProjectConfig>,
}

pub fn default_commands() -> Vec<CommandConfig> {
    vec![
        CommandConfig {
            name: "Claude Code".to_string(),
            command: "claude".to_string(),
            args: Vec::new(),
        },
        CommandConfig {
            name: "Codex".to_string(),
            command: "codex".to_string(),
            args: Vec::new(),
        },
    ]
}

impl Config {
    pub fn get_commands(&self) -> Vec<CommandConfig> {
        if self.commands.is_empty() {
            return default_commands();
        }
        self.commands.clone()
    }

    pub fn get_projects(&self) -> Vec<ProjectConfig> {
        self.projects.clone()
    }
}

pub fn app_name() -> &'static str {
    CURRENT_APP_NAME
}

pub fn base_config_dir() -> Result<PathBuf> {
    if let Some(value) = env_value("FORGE_CONFIG_DIR") {
        return normalize_dir(&value);
    }
    if let Some(value) = env_value("XDG_CONFIG_HOME") {
        return normalize_dir(&value);
    }

    Ok(user_home_dir()?.join(".config"))
}

pub fn app_config_dir() -> Result<PathBuf> {
    Ok(base_config_dir()?.join(app_name()))
}

pub fn config_path() -> Result<PathBuf> {
    Ok(app_config_dir()?.join("config.json"))
}

pub fn load() -> Result<(Config, bool)> {
    let path = config_path()?;
    let data = match std::fs::read(&path) {
        Ok(data) => data,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return load_from_legacy_config(),
        Err(err) => return Err(anyhow!("读取配置文件失败：{err}")),
    };

    let cfg = serde_json::from_slice::<Config>(&data)
        .map_err(|err| anyhow!("解析配置文件失败：{err}"))?;
    Ok((cfg, true))
}

pub fn save(mut cfg: Config) -> Result<PathBuf> {
    if cfg.root.trim().is_empty() {
        return Err(anyhow!("root 不能为空"));
    }

    if cfg.commands.is_empty() {
        cfg.commands = default_commands();
    }

    let path = config_path()?;
    let dir = path
        .parent()
        .ok_or_else(|| anyhow!("获取配置目录失败"))?
        .to_path_buf();
    std::fs::create_dir_all(&dir).map_err(|err| anyhow!("创建配置目录失败：{err}"))?;

    let mut data =
        serde_json::to_vec_pretty(&cfg).map_err(|err| anyhow!("序列化配置失败：{err}"))?;
    data.push(b'\n');

    let mut temp =
        tempfile::NamedTempFile::new_in(&dir).map_err(|err| anyhow!("创建临时文件失败：{err}"))?;
    temp.write_all(&data)
        .map_err(|err| anyhow!("写入临时配置失败：{err}"))?;
    temp.flush()
        .map_err(|err| anyhow!("关闭临时配置失败：{err}"))?;

    match temp.persist(&path) {
        Ok(_) => {}
        Err(err) => {
            std::fs::write(&path, &data)
                .map_err(|write_err| anyhow!("写入配置文件失败：{write_err}"))?;
            let _ = err.file.close();
        }
    }

    Ok(path)
}

pub fn save_root(root: &str) -> Result<PathBuf> {
    let normalized = root.trim();
    if normalized.is_empty() {
        return Err(anyhow!("root 不能为空"));
    }

    let (mut cfg, _) = load().unwrap_or_default();
    cfg.root = normalized.to_string();
    save(cfg)
}

fn load_from_legacy_config() -> Result<(Config, bool)> {
    let legacy_path = legacy_config_path()?;
    let data = match std::fs::read(&legacy_path) {
        Ok(data) => data,
        Err(_) => return Ok((Config::default(), false)),
    };

    let cfg = match serde_json::from_slice::<Config>(&data) {
        Ok(cfg) if !cfg.root.trim().is_empty() => cfg,
        _ => return Ok((Config::default(), false)),
    };

    let _ = save(cfg.clone());
    Ok((cfg, true))
}

fn legacy_config_path() -> Result<PathBuf> {
    Ok(legacy_base_config_dir()?
        .join(LEGACY_APP_NAME)
        .join("config.json"))
}

fn legacy_base_config_dir() -> Result<PathBuf> {
    if let Some(value) = env_value("AIDEV_CONFIG_DIR") {
        return normalize_dir(&value);
    }
    if let Some(value) = env_value("XDG_CONFIG_HOME") {
        return normalize_dir(&value);
    }

    Ok(user_home_dir()?.join(".config"))
}

fn normalize_dir(dir: &str) -> Result<PathBuf> {
    let expanded = expand_tilde(dir)?;
    absolute_clean(std::path::Path::new(&expanded))
        .map_err(|err| anyhow!("解析配置目录失败：{err}"))
}

fn env_value(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
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
    use std::fs;
    use std::path::PathBuf;

    use serial_test::serial;
    use temp_env::{with_var, with_vars};

    use super::{
        CURRENT_APP_NAME, CommandConfig, Config, LEGACY_APP_NAME, ProjectConfig, app_name,
        config_path, default_commands, load, save, save_root,
    };

    fn legacy_config_payload(root: &str) -> Vec<u8> {
        let mut payload = serde_json::to_vec(&serde_json::json!({ "root": root }))
            .expect("序列化 legacy 配置失败");
        payload.push(b'\n');
        payload
    }

    #[test]
    #[serial]
    fn test_config_path_uses_forge_config_dir() {
        let base = tempfile::tempdir().expect("创建临时目录失败");
        with_vars(
            vec![
                (
                    "FORGE_CONFIG_DIR",
                    Some(base.path().to_string_lossy().to_string()),
                ),
                (
                    "AIDEV_CONFIG_DIR",
                    Some(base.path().to_string_lossy().to_string()),
                ),
            ],
            || {
                let got = config_path().expect("ConfigPath 返回错误");
                let want = base.path().join(app_name()).join("config.json");
                assert_eq!(got, want, "ConfigPath 不符合预期");
            },
        );
    }

    #[test]
    #[serial]
    fn test_load_missing() {
        let base = tempfile::tempdir().expect("创建临时目录失败");
        with_vars(
            vec![
                (
                    "FORGE_CONFIG_DIR",
                    Some(base.path().to_string_lossy().to_string()),
                ),
                (
                    "AIDEV_CONFIG_DIR",
                    Some(base.path().to_string_lossy().to_string()),
                ),
            ],
            || {
                let (_, exists) = load().expect("Load 返回错误");
                assert!(!exists, "期望 exists=false");
            },
        );
    }

    #[test]
    #[serial]
    fn test_save_and_load() {
        let base = tempfile::tempdir().expect("创建临时目录失败");
        let root = tempfile::tempdir().expect("创建 root 目录失败");
        with_vars(
            vec![
                (
                    "FORGE_CONFIG_DIR",
                    Some(base.path().to_string_lossy().to_string()),
                ),
                (
                    "AIDEV_CONFIG_DIR",
                    Some(base.path().to_string_lossy().to_string()),
                ),
            ],
            || {
                let path = save(Config {
                    root: root.path().to_string_lossy().to_string(),
                    ..Config::default()
                })
                .expect("Save 返回错误");
                assert!(path.exists(), "配置文件未写入");

                let (cfg, exists) = load().expect("Load 返回错误");
                assert!(exists, "期望 exists=true");
                assert_eq!(
                    cfg.root,
                    root.path().to_string_lossy(),
                    "cfg.Root 不符合预期"
                );
                assert_eq!(cfg.commands.len(), 2, "Save 应自动写入 2 个默认命令");
                assert_eq!(cfg.commands[0].name, "Claude Code");
            },
        );
    }

    #[test]
    fn test_get_commands_default_when_empty() {
        let cfg = Config {
            root: "/tmp".to_string(),
            ..Config::default()
        };
        let commands = cfg.get_commands();

        assert_eq!(commands.len(), 2, "期望 2 个默认命令");
        assert_eq!(commands[0].name, "Claude Code");
        assert_eq!(commands[0].command, "claude");
        assert!(commands[0].args.is_empty(), "默认参数应为空数组");
        assert_eq!(commands[1].name, "Codex");
    }

    #[test]
    fn test_get_commands_custom_override() {
        let cfg = Config {
            root: "/tmp".to_string(),
            commands: vec![CommandConfig {
                name: "Aider".to_string(),
                command: "aider".to_string(),
                args: vec!["--auto-commits".to_string()],
            }],
            ..Config::default()
        };
        let commands = cfg.get_commands();

        assert_eq!(commands.len(), 1, "期望 1 个自定义命令");
        assert_eq!(commands[0].name, "Aider");
        assert_eq!(commands[0].command, "aider");
        assert_eq!(commands[0].args, vec!["--auto-commits"]);
    }

    #[test]
    #[serial]
    fn test_save_and_load_with_commands() {
        let base = tempfile::tempdir().expect("创建临时目录失败");
        let root = tempfile::tempdir().expect("创建 root 目录失败");
        let custom_cmds = vec![
            CommandConfig {
                name: "MyTool".to_string(),
                command: "mytool".to_string(),
                args: vec!["--flag1".to_string(), "--flag2".to_string()],
            },
            CommandConfig {
                name: "AnotherTool".to_string(),
                command: "another".to_string(),
                args: Vec::new(),
            },
        ];
        with_vars(
            vec![
                (
                    "FORGE_CONFIG_DIR",
                    Some(base.path().to_string_lossy().to_string()),
                ),
                (
                    "AIDEV_CONFIG_DIR",
                    Some(base.path().to_string_lossy().to_string()),
                ),
            ],
            || {
                let path = save(Config {
                    root: root.path().to_string_lossy().to_string(),
                    commands: custom_cmds.clone(),
                    ..Config::default()
                })
                .expect("Save 返回错误");
                assert!(path.exists(), "配置文件未写入");

                let (cfg, exists) = load().expect("Load 返回错误");
                assert!(exists, "期望 exists=true");
                assert_eq!(cfg.commands.len(), 2, "期望 2 个命令");
                assert_eq!(cfg.commands[0].name, "MyTool");
                assert_eq!(cfg.commands[1].command, "another");
            },
        );
    }

    #[test]
    #[serial]
    fn test_save_root_preserves_existing_commands() {
        let base = tempfile::tempdir().expect("创建临时目录失败");
        let root1 = tempfile::tempdir().expect("创建 root1 目录失败");
        let root2 = tempfile::tempdir().expect("创建 root2 目录失败");
        let custom_cmds = vec![CommandConfig {
            name: "MyTool".to_string(),
            command: "mytool".to_string(),
            args: vec!["--flag1".to_string()],
        }];
        with_vars(
            vec![
                (
                    "FORGE_CONFIG_DIR",
                    Some(base.path().to_string_lossy().to_string()),
                ),
                (
                    "AIDEV_CONFIG_DIR",
                    Some(base.path().to_string_lossy().to_string()),
                ),
            ],
            || {
                save(Config {
                    root: root1.path().to_string_lossy().to_string(),
                    commands: custom_cmds.clone(),
                    ..Config::default()
                })
                .expect("初始化配置失败");

                save_root(root2.path().to_string_lossy().as_ref()).expect("SaveRoot 返回错误");

                let (cfg, exists) = load().expect("Load 返回错误");
                assert!(exists, "期望 exists=true");
                assert_eq!(
                    cfg.root,
                    root2.path().to_string_lossy(),
                    "cfg.Root 不符合预期"
                );
                assert_eq!(cfg.commands.len(), 1, "期望保留 1 个命令");
                assert_eq!(cfg.commands[0].name, "MyTool");
                assert_eq!(cfg.commands[0].command, "mytool");
            },
        );
    }

    #[test]
    #[serial]
    fn test_load_migrate_from_legacy_aidev_config() {
        let forge_base = tempfile::tempdir().expect("创建 forge 配置目录失败");
        let legacy_base = tempfile::tempdir().expect("创建 legacy 配置目录失败");
        let root = tempfile::tempdir().expect("创建 root 目录失败");
        let legacy_path = legacy_base.path().join(LEGACY_APP_NAME).join("config.json");

        fs::create_dir_all(legacy_path.parent().expect("legacy 路径无父目录"))
            .expect("创建 legacy 配置目录失败");
        fs::write(
            &legacy_path,
            legacy_config_payload(root.path().to_string_lossy().as_ref()),
        )
        .expect("写入 legacy 配置失败");

        with_vars(
            vec![
                (
                    "FORGE_CONFIG_DIR",
                    Some(forge_base.path().to_string_lossy().to_string()),
                ),
                (
                    "AIDEV_CONFIG_DIR",
                    Some(legacy_base.path().to_string_lossy().to_string()),
                ),
            ],
            || {
                let (cfg, exists) = load().expect("Load 返回错误");
                assert!(exists, "期望 exists=true");
                assert_eq!(
                    cfg.root,
                    root.path().to_string_lossy(),
                    "cfg.Root 不符合预期"
                );

                let new_path = forge_base.path().join(CURRENT_APP_NAME).join("config.json");
                assert!(new_path.exists(), "期望迁移写入新配置文件");
            },
        );
    }

    #[test]
    #[serial]
    fn test_load_migrate_from_legacy_aidev_config_with_windows_style_root() {
        let forge_base = tempfile::tempdir().expect("创建 forge 配置目录失败");
        let legacy_base = tempfile::tempdir().expect("创建 legacy 配置目录失败");
        let legacy_path = legacy_base.path().join(LEGACY_APP_NAME).join("config.json");
        let windows_style_root = r"C:\Users\LittleBun\Projects\forge";

        fs::create_dir_all(legacy_path.parent().expect("legacy 路径无父目录"))
            .expect("创建 legacy 配置目录失败");
        fs::write(&legacy_path, legacy_config_payload(windows_style_root))
            .expect("写入 legacy 配置失败");

        with_vars(
            vec![
                (
                    "FORGE_CONFIG_DIR",
                    Some(forge_base.path().to_string_lossy().to_string()),
                ),
                (
                    "AIDEV_CONFIG_DIR",
                    Some(legacy_base.path().to_string_lossy().to_string()),
                ),
            ],
            || {
                let (cfg, exists) = load().expect("Load 返回错误");
                assert!(exists, "期望 exists=true");
                assert_eq!(cfg.root, windows_style_root, "cfg.Root 不符合预期");
            },
        );
    }

    #[test]
    fn test_get_projects_empty_when_none() {
        let cfg = Config {
            root: "/tmp".to_string(),
            ..Config::default()
        };
        let projects = cfg.get_projects();
        assert_eq!(projects.len(), 0, "期望 0 个项目书签");
    }

    #[test]
    #[serial]
    fn test_save_and_load_with_projects() {
        let base = tempfile::tempdir().expect("创建临时目录失败");
        let root = tempfile::tempdir().expect("创建 root 目录失败");
        let custom_projects = vec![
            ProjectConfig {
                name: "my-blog".to_string(),
                path: "~/Projects/my-blog".to_string(),
            },
            ProjectConfig {
                name: "todo-app".to_string(),
                path: "/Users/test/todo-app".to_string(),
            },
        ];

        with_vars(
            vec![
                (
                    "FORGE_CONFIG_DIR",
                    Some(base.path().to_string_lossy().to_string()),
                ),
                (
                    "AIDEV_CONFIG_DIR",
                    Some(base.path().to_string_lossy().to_string()),
                ),
            ],
            || {
                let path = save(Config {
                    root: root.path().to_string_lossy().to_string(),
                    projects: custom_projects.clone(),
                    ..Config::default()
                })
                .expect("Save 返回错误");
                assert!(path.exists(), "配置文件未写入");

                let (cfg, exists) = load().expect("Load 返回错误");
                assert!(exists, "期望 exists=true");
                assert_eq!(cfg.projects.len(), 2, "期望 2 个项目书签");
                assert_eq!(cfg.projects[0].name, "my-blog");
                assert_eq!(cfg.projects[1].path, "/Users/test/todo-app");
            },
        );
    }

    #[test]
    #[serial]
    fn test_save_root_preserves_existing_projects() {
        let base = tempfile::tempdir().expect("创建临时目录失败");
        let root1 = tempfile::tempdir().expect("创建 root1 目录失败");
        let root2 = tempfile::tempdir().expect("创建 root2 目录失败");
        let custom_projects = vec![ProjectConfig {
            name: "my-blog".to_string(),
            path: "~/Projects/my-blog".to_string(),
        }];

        with_vars(
            vec![
                (
                    "FORGE_CONFIG_DIR",
                    Some(base.path().to_string_lossy().to_string()),
                ),
                (
                    "AIDEV_CONFIG_DIR",
                    Some(base.path().to_string_lossy().to_string()),
                ),
            ],
            || {
                save(Config {
                    root: root1.path().to_string_lossy().to_string(),
                    projects: custom_projects.clone(),
                    ..Config::default()
                })
                .expect("初始化配置失败");

                save_root(root2.path().to_string_lossy().as_ref()).expect("SaveRoot 返回错误");

                let (cfg, exists) = load().expect("Load 返回错误");
                assert!(exists, "期望 exists=true");
                assert_eq!(
                    cfg.root,
                    root2.path().to_string_lossy(),
                    "cfg.Root 不符合预期"
                );
                assert_eq!(cfg.projects.len(), 1, "期望 1 个项目书签");
                assert_eq!(cfg.projects[0].name, "my-blog");
            },
        );
    }

    #[test]
    fn test_default_commands_have_empty_args() {
        let commands = default_commands();
        assert_eq!(commands.len(), 2, "默认命令数量不符合预期");
        assert!(commands[0].args.is_empty(), "Claude 默认参数应为空");
        assert!(commands[1].args.is_empty(), "Codex 默认参数应为空");
    }

    #[test]
    #[serial]
    fn test_load_invalid_json_returns_error() {
        let base = tempfile::tempdir().expect("创建临时目录失败");
        let app_dir = base.path().join(CURRENT_APP_NAME);
        fs::create_dir_all(&app_dir).expect("创建应用配置目录失败");
        fs::write(app_dir.join("config.json"), "{invalid-json").expect("写入损坏配置失败");

        with_var(
            "FORGE_CONFIG_DIR",
            Some(base.path().to_string_lossy().to_string()),
            || {
                let err = load().expect_err("损坏配置应返回错误");
                assert!(err.to_string().contains("解析配置文件失败"));
            },
        );
    }

    #[test]
    #[serial]
    fn test_save_writes_trailing_newline() {
        let base = tempfile::tempdir().expect("创建临时目录失败");
        let root = tempfile::tempdir().expect("创建 root 目录失败");
        with_vars(
            vec![
                (
                    "FORGE_CONFIG_DIR",
                    Some(base.path().to_string_lossy().to_string()),
                ),
                (
                    "AIDEV_CONFIG_DIR",
                    Some(base.path().to_string_lossy().to_string()),
                ),
            ],
            || {
                let path = save(Config {
                    root: root.path().to_string_lossy().to_string(),
                    ..Config::default()
                })
                .expect("Save 返回错误");
                let content = fs::read_to_string(path).expect("读取配置文件失败");
                assert!(content.ends_with('\n'), "配置文件应以换行结尾");
            },
        );
    }

    #[test]
    fn test_constants_match_expected_app_names() {
        assert_eq!(app_name(), CURRENT_APP_NAME);
        assert_eq!(LEGACY_APP_NAME, "aidev");
        assert_eq!(PathBuf::from(CURRENT_APP_NAME), PathBuf::from("forge"));
    }
}
