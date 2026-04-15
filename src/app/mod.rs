use std::ffi::OsString;
use std::io::{IsTerminal, Write};
use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};

use crate::config::{self, CommandConfig, Config, ProjectConfig};
use crate::scan::{self, Dir, Options};
use crate::ui::{self, RootOption, WorkspaceChoice, WorkspaceResult};

const EXIT_OK: i32 = 0;
const EXIT_ERROR: i32 = 1;
const EXIT_USAGE: i32 = 2;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn build_commit() -> &'static str {
    option_env!("FORGE_GIT_COMMIT").unwrap_or("none")
}

fn build_date() -> &'static str {
    option_env!("FORGE_BUILD_DATE").unwrap_or("unknown")
}

pub trait Services {
    fn print_banner(&self, stderr: &mut dyn Write, version: &str) -> Result<()>;
    fn config_load(&self) -> Result<(Config, bool)>;
    fn config_path(&self) -> Result<PathBuf>;
    fn config_save(&self, cfg: Config) -> Result<PathBuf>;
    fn config_save_root(&self, root: &str) -> Result<PathBuf>;
    fn normalize_root(&self, root: &str) -> Result<String>;
    fn detect_default_root(&self) -> String;
    fn list_dirs(&self, root: &str, options: Options) -> Result<Vec<Dir>>;
    fn run_command(&self, cmd_name: &str, cmd_args: &[String], work_dir: &str) -> Result<()>;
    fn select_workspace(
        &self,
        saved_root: &str,
        projects: &[ProjectConfig],
    ) -> Result<(WorkspaceResult, bool)>;
    fn select_root(
        &self,
        label: &str,
        options: &[RootOption],
        default_input: &str,
    ) -> Result<(String, bool)>;
    fn select_dir(&self, dirs: &[Dir]) -> Result<(Dir, bool)>;
    fn select_command(&self, commands: &[CommandConfig]) -> Result<(CommandConfig, bool)>;
    fn env_var(&self, name: &str) -> Option<String>;
    fn is_stdin_terminal(&self) -> bool;
    fn current_dir(&self) -> Result<String>;
    fn home_dir(&self) -> Result<String>;
}

struct RealServices;

impl Services for RealServices {
    fn print_banner(&self, stderr: &mut dyn Write, version: &str) -> Result<()> {
        ui::print_banner(stderr, version)
    }

    fn config_load(&self) -> Result<(Config, bool)> {
        config::load()
    }

    fn config_path(&self) -> Result<PathBuf> {
        config::config_path()
    }

    fn config_save(&self, cfg: Config) -> Result<PathBuf> {
        config::save(cfg)
    }

    fn config_save_root(&self, root: &str) -> Result<PathBuf> {
        config::save_root(root)
    }

    fn normalize_root(&self, root: &str) -> Result<String> {
        crate::pathutil::normalize_root(root)
    }

    fn detect_default_root(&self) -> String {
        crate::pathutil::detect_default_root()
    }

    fn list_dirs(&self, root: &str, options: Options) -> Result<Vec<Dir>> {
        scan::list_dirs(root, options)
    }

    fn run_command(&self, cmd_name: &str, cmd_args: &[String], work_dir: &str) -> Result<()> {
        crate::runner::run_command(cmd_name, cmd_args, work_dir)
    }

    fn select_workspace(
        &self,
        saved_root: &str,
        projects: &[ProjectConfig],
    ) -> Result<(WorkspaceResult, bool)> {
        ui::select_workspace(saved_root, projects)
    }

    fn select_root(
        &self,
        label: &str,
        options: &[RootOption],
        default_input: &str,
    ) -> Result<(String, bool)> {
        ui::select_root(label, options, default_input)
    }

    fn select_dir(&self, dirs: &[Dir]) -> Result<(Dir, bool)> {
        ui::select_dir(dirs)
    }

    fn select_command(&self, commands: &[CommandConfig]) -> Result<(CommandConfig, bool)> {
        ui::select_command(commands)
    }

    fn env_var(&self, name: &str) -> Option<String> {
        std::env::var(name)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    }

    fn is_stdin_terminal(&self) -> bool {
        std::io::stdin().is_terminal()
    }

    fn current_dir(&self) -> Result<String> {
        Ok(std::env::current_dir()?.to_string_lossy().to_string())
    }

    fn home_dir(&self) -> Result<String> {
        std::env::var("HOME").map_err(|err| anyhow!("获取用户 HOME 目录失败：{err}"))
    }
}

pub fn run(args: Vec<OsString>, stdout: &mut dyn Write, stderr: &mut dyn Write) -> i32 {
    run_with_services(args, stdout, stderr, &RealServices)
}

fn run_with_services(
    args: Vec<OsString>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    services: &dyn Services,
) -> i32 {
    let program_path = args
        .first()
        .cloned()
        .unwrap_or_else(|| OsString::from("forge"));
    let program_name = Path::new(&program_path)
        .file_name()
        .unwrap_or(program_path.as_os_str())
        .to_string_lossy()
        .to_string();
    let raw_args: Vec<String> = args
        .iter()
        .skip(1)
        .map(|arg| arg.to_string_lossy().to_string())
        .collect();

    if let Some(first) = raw_args.first() {
        match first.as_str() {
            "config" | "cfg" => {
                return run_config_command(&program_name, &raw_args[1..], stdout, stderr, services);
            }
            "root" => {
                return run_root_command(&program_name, &raw_args[1..], stdout, stderr, services);
            }
            "version" | "-v" | "--version" => return run_version_command(&program_name, stdout),
            _ => {}
        }
    }

    if let Err(err) = services.print_banner(stderr, VERSION) {
        let _ = writeln!(stderr, "打印 Banner 失败：{err}");
        return EXIT_ERROR;
    }

    let parsed = match parse_main_args(&raw_args) {
        Ok(parsed) => parsed,
        Err(ParseOutcome::Help) => {
            let _ = print_main_usage(&program_name, stderr);
            return EXIT_OK;
        }
        Err(ParseOutcome::Error(message)) => {
            let _ = writeln!(stderr, "参数解析失败：{message}");
            return EXIT_USAGE;
        }
    };

    if parsed.version_flag {
        return run_version_command(&program_name, stdout);
    }

    if let Some(explicit_root) = parsed
        .explicit_root
        .filter(|value| !value.trim().is_empty())
    {
        return run_with_explicit_root(&explicit_root, stdout, stderr, services);
    }

    run_interactive_workspace(&program_name, stdout, stderr, services)
}

fn run_with_explicit_root(
    root: &str,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    services: &dyn Services,
) -> i32 {
    let normalized_root = match services.normalize_root(root) {
        Ok(root) => root,
        Err(err) => {
            let _ = writeln!(stderr, "根目录无效：{err}");
            return EXIT_USAGE;
        }
    };

    if services.env_var("FORGE_ROOT").is_none() && services.env_var("AIDEV_ROOT").is_none() {
        let load_result = services.config_load();
        let exists = match load_result {
            Ok((_, exists)) => exists,
            Err(err) => {
                let _ = writeln!(stderr, "读取本地配置失败（将尝试覆盖写入）：{err}");
                false
            }
        };

        if !exists {
            match services.config_save(Config {
                root: normalized_root.clone(),
                ..Config::default()
            }) {
                Ok(path) => {
                    let _ = writeln!(
                        stderr,
                        "已保存默认根目录：{}（{}）",
                        normalized_root,
                        path.to_string_lossy()
                    );
                }
                Err(err) => {
                    let _ = writeln!(stderr, "保存默认根目录失败：{err}");
                }
            }
        }
    }

    scan_and_select_dir(&normalized_root, stdout, stderr, services)
}

fn run_interactive_workspace(
    program_name: &str,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    services: &dyn Services,
) -> i32 {
    let env_root = services
        .env_var("FORGE_ROOT")
        .or_else(|| services.env_var("AIDEV_ROOT"));
    if let Some(env_root) = env_root {
        let normalized_root = match services.normalize_root(&env_root) {
            Ok(root) => root,
            Err(err) => {
                let _ = writeln!(stderr, "环境变量根目录无效：{err}");
                return EXIT_ERROR;
            }
        };
        return scan_and_select_dir(&normalized_root, stdout, stderr, services);
    }

    let (cfg, _cfg_exists) = match services.config_load() {
        Ok(value) => value,
        Err(err) => {
            let _ = writeln!(stderr, "读取本地配置失败：{err}");
            (Config::default(), false)
        }
    };

    let saved_root = cfg.root.trim().to_string();
    let projects = cfg.get_projects();
    if saved_root.is_empty() && projects.is_empty() {
        if !services.is_stdin_terminal() {
            let _ = writeln!(
                stderr,
                "当前环境非交互式终端，请通过 {program_name} --root 或 {program_name} config set-root 设置根目录"
            );
            return EXIT_ERROR;
        }
        let root = match select_and_persist_root(
            "首次使用：请选择默认根目录（只扫描一级子目录）",
            program_name,
            stderr,
            services,
        ) {
            Ok(Some(root)) => root,
            Ok(None) => return EXIT_OK,
            Err(err) => {
                let _ = writeln!(stderr, "选择根目录失败：{err}");
                return EXIT_ERROR;
            }
        };
        let normalized_root = match services.normalize_root(&root) {
            Ok(root) => root,
            Err(err) => {
                let _ = writeln!(stderr, "根目录无效：{err}");
                return EXIT_ERROR;
            }
        };
        return scan_and_select_dir(&normalized_root, stdout, stderr, services);
    }

    let _ = writeln!(stderr, "{}", ui::step_line("  Step 1/3", "选择工作区"));
    let _ = writeln!(stderr);
    let (result, canceled) = match services.select_workspace(&saved_root, &projects) {
        Ok(result) => result,
        Err(err) => {
            let _ = writeln!(stderr, "工作区选择失败：{err}");
            return EXIT_ERROR;
        }
    };
    if canceled {
        return EXIT_OK;
    }

    let project_dir = match result.choice {
        WorkspaceChoice::CurrentDir => {
            let _ = writeln!(stderr, "{}", ui::selected_line("当前目录"));
            let _ = writeln!(stderr, "{}", ui::subtitle_line(&result.project_path));
            let _ = writeln!(stderr);
            result.project_path
        }
        WorkspaceChoice::Project => {
            let project_dir = match services.normalize_root(&result.project_path) {
                Ok(path) => path,
                Err(err) => {
                    let _ = writeln!(stderr, "项目路径无效：{err}");
                    return EXIT_ERROR;
                }
            };
            let label = Path::new(&project_dir)
                .file_name()
                .map(|value| value.to_string_lossy().to_string())
                .unwrap_or_else(|| project_dir.clone());
            let _ = writeln!(stderr, "{}", ui::selected_line(&label));
            let _ = writeln!(stderr, "{}", ui::subtitle_line(&project_dir));
            let _ = writeln!(stderr);
            project_dir
        }
        WorkspaceChoice::BrowseRoot => {
            let normalized_root = match services.normalize_root(&saved_root) {
                Ok(root) => root,
                Err(err) => {
                    let _ = writeln!(stderr, "已保存的根目录无效：{err}\n请重新选择根目录…");
                    let root = match select_and_persist_root(
                        "请选择项目根目录（只扫描一级子目录）",
                        program_name,
                        stderr,
                        services,
                    ) {
                        Ok(Some(root)) => root,
                        Ok(None) => return EXIT_OK,
                        Err(err) => {
                            let _ = writeln!(stderr, "选择根目录失败：{err}");
                            return EXIT_ERROR;
                        }
                    };
                    match services.normalize_root(&root) {
                        Ok(root) => root,
                        Err(err) => {
                            let _ = writeln!(stderr, "根目录无效：{err}");
                            return EXIT_ERROR;
                        }
                    }
                }
            };
            return scan_and_select_dir(&normalized_root, stdout, stderr, services);
        }
        WorkspaceChoice::NewRoot => {
            if !services.is_stdin_terminal() {
                let _ = writeln!(
                    stderr,
                    "当前环境非交互式终端，请通过 {program_name} --root 或 {program_name} config set-root 设置根目录"
                );
                return EXIT_ERROR;
            }
            let root = match select_and_persist_root(
                "选择新的根目录（只扫描一级子目录）",
                program_name,
                stderr,
                services,
            ) {
                Ok(Some(root)) => root,
                Ok(None) => return EXIT_OK,
                Err(err) => {
                    let _ = writeln!(stderr, "选择根目录失败：{err}");
                    return EXIT_ERROR;
                }
            };
            let normalized_root = match services.normalize_root(&root) {
                Ok(root) => root,
                Err(err) => {
                    let _ = writeln!(stderr, "根目录无效：{err}");
                    return EXIT_ERROR;
                }
            };
            return scan_and_select_dir(&normalized_root, stdout, stderr, services);
        }
    };

    select_command_and_run(&project_dir, stdout, stderr, services)
}

fn scan_and_select_dir(
    root: &str,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    services: &dyn Services,
) -> i32 {
    let dirs = match services.list_dirs(
        root,
        Options {
            ignore_names: Some(scan::default_ignore_names()),
        },
    ) {
        Ok(dirs) => dirs,
        Err(err) => {
            let _ = writeln!(stderr, "扫描目录失败：{err}");
            return EXIT_ERROR;
        }
    };
    if dirs.is_empty() {
        let _ = writeln!(stderr, "根目录下没有可选项目目录：{root}");
        return EXIT_ERROR;
    }

    let _ = writeln!(stderr, "{}", ui::step_line("  Step 2/3", "选择项目目录"));
    let _ = writeln!(stderr);
    let (selected_dir, canceled) = match services.select_dir(&dirs) {
        Ok(result) => result,
        Err(err) => {
            let _ = writeln!(stderr, "选择目录失败：{err}");
            return EXIT_ERROR;
        }
    };
    if canceled {
        return EXIT_OK;
    }

    let _ = writeln!(stdout, "{}", ui::selected_line(&selected_dir.name));
    let _ = writeln!(stdout, "{}", ui::subtitle_line(&selected_dir.path));
    let _ = writeln!(stdout);

    select_command_and_run(&selected_dir.path, stdout, stderr, services)
}

fn select_command_and_run(
    project_dir: &str,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    services: &dyn Services,
) -> i32 {
    let commands = match services.config_load() {
        Ok((cfg, _)) => cfg.get_commands(),
        Err(err) => {
            let _ = writeln!(stderr, "读取本地配置失败：{err}");
            let _ = writeln!(stderr, "将使用内置默认命令。");
            config::default_commands()
        }
    };

    let _ = writeln!(stderr, "{}", ui::step_line("  Step 3/3", "选择启动模式"));
    let _ = writeln!(stderr);
    let (selected_cmd, canceled) = match services.select_command(&commands) {
        Ok(result) => result,
        Err(err) => {
            let _ = writeln!(stderr, "选择命令失败：{err}");
            return EXIT_ERROR;
        }
    };
    if canceled {
        return EXIT_OK;
    }

    let _ = writeln!(stdout, "{}", ui::selected_line(&selected_cmd.name));
    let desc = if selected_cmd.args.is_empty() {
        selected_cmd.command.clone()
    } else {
        format!("{} {}", selected_cmd.command, selected_cmd.args.join(" "))
    };
    let _ = writeln!(stdout, "{}", ui::subtitle_line(&desc));
    let _ = writeln!(stdout);

    if let Err(err) = services.run_command(&selected_cmd.command, &selected_cmd.args, project_dir) {
        let _ = writeln!(stderr, "{err}");
        return EXIT_ERROR;
    }
    EXIT_OK
}

fn select_and_persist_root(
    label: &str,
    program_name: &str,
    stderr: &mut dyn Write,
    services: &dyn Services,
) -> Result<Option<String>> {
    if !services.is_stdin_terminal() {
        return Err(anyhow!(
            "当前环境非交互式终端，请通过 {program_name} --root 或 {program_name} config set-root 设置根目录"
        ));
    }

    let (options, default_input) = build_root_options(services)?;
    loop {
        let (selected, canceled) = services.select_root(label, &options, &default_input)?;
        if canceled {
            return Ok(None);
        }

        let normalized = match services.normalize_root(&selected) {
            Ok(root) => root,
            Err(err) => {
                let _ = writeln!(stderr, "根目录无效：{err}");
                continue;
            }
        };

        match services.config_save_root(&normalized) {
            Ok(path) => {
                let _ = writeln!(
                    stderr,
                    "已保存默认根目录：{}（{}）",
                    normalized,
                    path.to_string_lossy()
                );
            }
            Err(err) => {
                let _ = writeln!(stderr, "保存默认根目录失败：{err}");
            }
        }
        return Ok(Some(normalized));
    }
}

fn build_root_options(services: &dyn Services) -> Result<(Vec<RootOption>, String)> {
    let default_root = services.detect_default_root();
    let default_input = if default_root != "." {
        default_root.clone()
    } else {
        "~/Projects".to_string()
    };

    let mut options = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut add = |label: String, value: String| {
        if value.is_empty() || !seen.insert(value.clone()) {
            return;
        }
        options.push(RootOption {
            label,
            value,
            is_manual: false,
        });
    };

    if default_root != "." {
        add(format!("推荐：{default_root}"), default_root.clone());
    }

    if let Ok(home) = services.home_dir() {
        let projects = Path::new(&home).join("Projects");
        if projects.is_dir() {
            add(
                "~/Projects".to_string(),
                projects.to_string_lossy().to_string(),
            );
        }
        let idea_projects = Path::new(&home).join("IdeaProjects");
        if idea_projects.is_dir() {
            add(
                "~/IdeaProjects".to_string(),
                idea_projects.to_string_lossy().to_string(),
            );
        }
    }

    add("当前目录（.）".to_string(), ".".to_string());
    options.push(RootOption {
        label: "手动输入路径...".to_string(),
        value: String::new(),
        is_manual: true,
    });

    Ok((options, default_input))
}

fn run_config_command(
    program_name: &str,
    args: &[String],
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    services: &dyn Services,
) -> i32 {
    if args.is_empty() || args[0] == "show" {
        return run_config_show(program_name, stdout, stderr, services);
    }

    match args[0].as_str() {
        "path" => run_config_path(stdout, stderr, services),
        "set-root" => run_config_set_root(program_name, &args[1..], stdout, stderr, services),
        "help" | "-h" | "--help" => {
            let _ = print_config_usage(program_name, stderr);
            EXIT_USAGE
        }
        _ => {
            let _ = print_config_usage(program_name, stderr);
            EXIT_USAGE
        }
    }
}

fn run_config_show(
    program_name: &str,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    services: &dyn Services,
) -> i32 {
    let path = match services.config_path() {
        Ok(path) => path,
        Err(err) => {
            let _ = writeln!(stderr, "获取配置路径失败：{err}");
            return EXIT_ERROR;
        }
    };

    let (cfg, exists) = match services.config_load() {
        Ok(value) => value,
        Err(err) => {
            let _ = writeln!(stderr, "读取配置失败：{err}");
            return EXIT_ERROR;
        }
    };

    let _ = writeln!(stdout, "配置文件: {}", path.to_string_lossy());
    if !exists || cfg.root.trim().is_empty() {
        let _ = writeln!(stdout, "默认 root: 未设置");
        let _ = writeln!(
            stdout,
            "设置方法: {program_name} config set-root \"~/Projects\""
        );
    } else {
        let _ = writeln!(stdout, "默认 root: {}", cfg.root);
    }

    let commands = cfg.get_commands();
    if cfg.commands.is_empty() {
        let _ = writeln!(stdout, "\n启动命令（内置默认）:");
    } else {
        let _ = writeln!(stdout, "\n启动命令（自定义）:");
    }
    for (index, command) in commands.iter().enumerate() {
        let args = command.args.join(" ");
        let _ = writeln!(
            stdout,
            "  {}. {} → {} {}",
            index + 1,
            command.name,
            command.command,
            args
        );
    }

    EXIT_OK
}

fn run_config_path(stdout: &mut dyn Write, stderr: &mut dyn Write, services: &dyn Services) -> i32 {
    match services.config_path() {
        Ok(path) => {
            let _ = writeln!(stdout, "{}", path.to_string_lossy());
            EXIT_OK
        }
        Err(err) => {
            let _ = writeln!(stderr, "获取配置路径失败：{err}");
            EXIT_ERROR
        }
    }
}

fn run_config_set_root(
    program_name: &str,
    args: &[String],
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    services: &dyn Services,
) -> i32 {
    if args.is_empty() {
        let root = match select_and_persist_root(
            "请选择默认根目录（只扫描一级子目录）",
            program_name,
            stderr,
            services,
        ) {
            Ok(Some(root)) => root,
            Ok(None) => return EXIT_OK,
            Err(err) => {
                let _ = writeln!(stderr, "设置 root 失败：{err}");
                return EXIT_ERROR;
            }
        };
        let _ = writeln!(stdout, "默认 root 已更新：{root}");
        return EXIT_OK;
    }

    let normalized = match services.normalize_root(&args[0]) {
        Ok(root) => root,
        Err(err) => {
            let _ = writeln!(stderr, "根目录无效：{err}");
            return EXIT_USAGE;
        }
    };

    match services.config_save_root(&normalized) {
        Ok(path) => {
            let _ = writeln!(stdout, "默认 root 已更新：{normalized}");
            let _ = writeln!(stdout, "配置文件: {}", path.to_string_lossy());
            EXIT_OK
        }
        Err(err) => {
            let _ = writeln!(stderr, "写入配置失败：{err}");
            EXIT_ERROR
        }
    }
}

fn run_root_command(
    program_name: &str,
    args: &[String],
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    services: &dyn Services,
) -> i32 {
    if args.is_empty() {
        return run_config_show(program_name, stdout, stderr, services);
    }

    match args[0].as_str() {
        "set" => run_config_set_root(program_name, &args[1..], stdout, stderr, services),
        "help" | "-h" | "--help" => {
            let _ = print_root_usage(program_name, stderr);
            EXIT_USAGE
        }
        _ => {
            let _ = print_root_usage(program_name, stderr);
            EXIT_USAGE
        }
    }
}

fn run_version_command(program_name: &str, stdout: &mut dyn Write) -> i32 {
    let _ = writeln!(stdout, "{program_name} {VERSION}");
    let _ = writeln!(stdout, "commit:  {}", build_commit());
    let _ = writeln!(stdout, "built:   {}", build_date());
    let _ = writeln!(stdout, "rust:    unknown");
    let _ = writeln!(
        stdout,
        "os/arch: {}/{}",
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    EXIT_OK
}

fn print_main_usage(program_name: &str, stderr: &mut dyn Write) -> std::io::Result<()> {
    writeln!(stderr, "用法: {program_name} [--root <dir>] [<dir>]")?;
    writeln!(stderr)?;
    writeln!(stderr, "参数：")?;
    writeln!(stderr, "  -root string")?;
    writeln!(
        stderr,
        "    \t扫描根目录（可选；未指定时读取本地配置；首次使用会引导选择并持久化）"
    )?;
    writeln!(stderr, "  -v\t输出版本信息并退出（简写）")?;
    writeln!(stderr, "  -version")?;
    writeln!(stderr, "    \t输出版本信息并退出")?;
    writeln!(stderr)?;
    writeln!(stderr, "说明：")?;
    writeln!(stderr, "  - 仅扫描指定根目录的一级子文件夹（Depth=1）")?;
    writeln!(
        stderr,
        "  - 强制忽略以 . 开头的目录（如 .git/.idea/.vscode）"
    )?;
    writeln!(stderr, "  - 额外忽略：node_modules/target/dist/vendor")?;
    writeln!(
        stderr,
        "  - 默认 root 会持久化到本地配置文件（可通过子命令修改）："
    )?;
    writeln!(
        stderr,
        "      {program_name} config set-root \"~/Projects\""
    )?;
    writeln!(stderr)?;
    writeln!(stderr, "示例：")?;
    writeln!(stderr, "  {program_name} --root \"~/Projects\"")?;
    writeln!(stderr, "  {program_name} \"~/IdeaProjects\"")
}

fn print_config_usage(program_name: &str, stderr: &mut dyn Write) -> std::io::Result<()> {
    writeln!(stderr, "用法: {program_name} config <command>")?;
    writeln!(stderr)?;
    writeln!(stderr, "命令：")?;
    writeln!(stderr, "  show              显示当前配置（默认）")?;
    writeln!(stderr, "  path              输出配置文件路径")?;
    writeln!(
        stderr,
        "  set-root [<dir>]  设置默认 root（不带参数则交互式选择）"
    )?;
    writeln!(stderr)?;
    writeln!(stderr, "示例：")?;
    writeln!(stderr, "  {program_name} config")?;
    writeln!(stderr, "  {program_name} config set-root \"~/Projects\"")
}

fn print_root_usage(program_name: &str, stderr: &mut dyn Write) -> std::io::Result<()> {
    writeln!(stderr, "用法: {program_name} root <command>")?;
    writeln!(stderr)?;
    writeln!(stderr, "命令：")?;
    writeln!(stderr, "  (空)              显示当前默认 root")?;
    writeln!(
        stderr,
        "  set [<dir>]       设置默认 root（不带参数则交互式选择）"
    )?;
    writeln!(stderr)?;
    writeln!(stderr, "示例：")?;
    writeln!(stderr, "  {program_name} root")?;
    writeln!(stderr, "  {program_name} root set \"~/Projects\"")
}

#[derive(Debug, Clone)]
struct ParsedMainArgs {
    explicit_root: Option<String>,
    version_flag: bool,
}

enum ParseOutcome {
    Help,
    Error(String),
}

fn parse_main_args(args: &[String]) -> std::result::Result<ParsedMainArgs, ParseOutcome> {
    let mut explicit_root = None;
    let mut version_flag = false;
    let mut positionals = Vec::new();
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--help" | "-h" | "-help" => return Err(ParseOutcome::Help),
            "--version" | "-v" => {
                version_flag = true;
                index += 1;
            }
            "--root" | "-root" => {
                let Some(value) = args.get(index + 1) else {
                    return Err(ParseOutcome::Error("flag 需要一个参数: --root".to_string()));
                };
                explicit_root = Some(value.trim().to_string());
                index += 2;
            }
            value if value.starts_with("--root=") => {
                explicit_root = Some(value.trim_start_matches("--root=").trim().to_string());
                index += 1;
            }
            value if value.starts_with('-') => {
                return Err(ParseOutcome::Error(format!("flag 未定义: {value}")));
            }
            value => {
                positionals.push(value.to_string());
                index += 1;
            }
        }
    }

    if explicit_root.is_none() && !positionals.is_empty() {
        explicit_root = Some(positionals[0].clone());
    }

    Ok(ParsedMainArgs {
        explicit_root,
        version_flag,
    })
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::{HashMap, VecDeque};
    use std::path::PathBuf;

    use anyhow::{Result, anyhow};

    use super::*;

    struct FakeServices {
        env: HashMap<String, String>,
        is_terminal: bool,
        home_dir: String,
        current_dir: String,
        config_load_results: RefCell<VecDeque<Result<(Config, bool)>>>,
        config_save_results: RefCell<VecDeque<Result<PathBuf>>>,
        config_save_root_results: RefCell<VecDeque<Result<PathBuf>>>,
        normalize_root_results: RefCell<VecDeque<Result<String>>>,
        list_dirs_results: RefCell<VecDeque<Result<Vec<Dir>>>>,
        select_workspace_results: RefCell<VecDeque<Result<(WorkspaceResult, bool)>>>,
        select_root_results: RefCell<VecDeque<Result<(String, bool)>>>,
        select_dir_results: RefCell<VecDeque<Result<(Dir, bool)>>>,
        select_command_results: RefCell<VecDeque<Result<(CommandConfig, bool)>>>,
        run_command_calls: RefCell<Vec<(String, Vec<String>, String)>>,
        run_command_error: RefCell<Option<String>>,
        printed_banner: RefCell<usize>,
        config_path_value: RefCell<PathBuf>,
        config_path_error: RefCell<Option<String>>,
        detect_default_root: String,
        listed_roots: RefCell<Vec<String>>,
    }

    impl FakeServices {
        fn new() -> Self {
            Self {
                env: HashMap::new(),
                is_terminal: true,
                home_dir: "/home/test".to_string(),
                current_dir: "/current".to_string(),
                config_load_results: RefCell::new(VecDeque::new()),
                config_save_results: RefCell::new(VecDeque::new()),
                config_save_root_results: RefCell::new(VecDeque::new()),
                normalize_root_results: RefCell::new(VecDeque::new()),
                list_dirs_results: RefCell::new(VecDeque::new()),
                select_workspace_results: RefCell::new(VecDeque::new()),
                select_root_results: RefCell::new(VecDeque::new()),
                select_dir_results: RefCell::new(VecDeque::new()),
                select_command_results: RefCell::new(VecDeque::new()),
                run_command_calls: RefCell::new(Vec::new()),
                run_command_error: RefCell::new(None),
                printed_banner: RefCell::new(0),
                config_path_value: RefCell::new(PathBuf::from("/cfg/forge/config.json")),
                config_path_error: RefCell::new(None),
                detect_default_root: ".".to_string(),
                listed_roots: RefCell::new(Vec::new()),
            }
        }
    }

    impl Services for FakeServices {
        fn print_banner(&self, _stderr: &mut dyn Write, _version: &str) -> Result<()> {
            *self.printed_banner.borrow_mut() += 1;
            Ok(())
        }

        fn config_load(&self) -> Result<(Config, bool)> {
            self.config_load_results
                .borrow_mut()
                .pop_front()
                .unwrap_or_else(|| Ok((Config::default(), false)))
        }

        fn config_path(&self) -> Result<PathBuf> {
            if let Some(message) = self.config_path_error.borrow().clone() {
                return Err(anyhow!(message));
            }
            Ok(self.config_path_value.borrow().clone())
        }

        fn config_save(&self, _cfg: Config) -> Result<PathBuf> {
            self.config_save_results
                .borrow_mut()
                .pop_front()
                .unwrap_or_else(|| Ok(PathBuf::from("/cfg/forge/config.json")))
        }

        fn config_save_root(&self, _root: &str) -> Result<PathBuf> {
            self.config_save_root_results
                .borrow_mut()
                .pop_front()
                .unwrap_or_else(|| Ok(PathBuf::from("/cfg/forge/config.json")))
        }

        fn normalize_root(&self, _root: &str) -> Result<String> {
            self.normalize_root_results
                .borrow_mut()
                .pop_front()
                .unwrap_or_else(|| Ok(_root.to_string()))
        }

        fn detect_default_root(&self) -> String {
            self.detect_default_root.clone()
        }

        fn list_dirs(&self, root: &str, _options: Options) -> Result<Vec<Dir>> {
            self.listed_roots.borrow_mut().push(root.to_string());
            self.list_dirs_results
                .borrow_mut()
                .pop_front()
                .unwrap_or_else(|| Ok(Vec::new()))
        }

        fn run_command(&self, cmd_name: &str, cmd_args: &[String], work_dir: &str) -> Result<()> {
            self.run_command_calls.borrow_mut().push((
                cmd_name.to_string(),
                cmd_args.to_vec(),
                work_dir.to_string(),
            ));
            if let Some(message) = self.run_command_error.borrow().clone() {
                return Err(anyhow!(message));
            }
            Ok(())
        }

        fn select_workspace(
            &self,
            _saved_root: &str,
            _projects: &[ProjectConfig],
        ) -> Result<(WorkspaceResult, bool)> {
            self.select_workspace_results
                .borrow_mut()
                .pop_front()
                .unwrap_or_else(|| Err(anyhow!("未设置工作区选择结果")))
        }

        fn select_root(
            &self,
            _label: &str,
            _options: &[RootOption],
            _default_input: &str,
        ) -> Result<(String, bool)> {
            self.select_root_results
                .borrow_mut()
                .pop_front()
                .unwrap_or_else(|| Err(anyhow!("未设置根目录选择结果")))
        }

        fn select_dir(&self, _dirs: &[Dir]) -> Result<(Dir, bool)> {
            self.select_dir_results
                .borrow_mut()
                .pop_front()
                .unwrap_or_else(|| Err(anyhow!("未设置目录选择结果")))
        }

        fn select_command(&self, _commands: &[CommandConfig]) -> Result<(CommandConfig, bool)> {
            self.select_command_results
                .borrow_mut()
                .pop_front()
                .unwrap_or_else(|| Err(anyhow!("未设置命令选择结果")))
        }

        fn env_var(&self, name: &str) -> Option<String> {
            self.env.get(name).cloned()
        }

        fn is_stdin_terminal(&self) -> bool {
            self.is_terminal
        }

        fn current_dir(&self) -> Result<String> {
            Ok(self.current_dir.clone())
        }

        fn home_dir(&self) -> Result<String> {
            Ok(self.home_dir.clone())
        }
    }

    #[test]
    fn test_parse_main_args_help() {
        let result = parse_main_args(&["--help".to_string()]);
        assert!(matches!(result, Err(ParseOutcome::Help)));
    }

    #[test]
    fn test_run_version_short_circuits_banner() {
        let services = FakeServices::new();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let exit_code = run_with_services(
            vec![OsString::from("forge"), OsString::from("--version")],
            &mut stdout,
            &mut stderr,
            &services,
        );

        assert_eq!(exit_code, EXIT_OK);
        assert_eq!(
            *services.printed_banner.borrow(),
            0,
            "version 子命令不应打印 Banner"
        );
        let output = String::from_utf8(stdout).unwrap();
        assert!(output.contains(&format!("forge {}", env!("CARGO_PKG_VERSION"))));
    }

    #[test]
    fn test_run_with_explicit_root_scans_and_runs_selected_command() {
        let services = FakeServices::new();
        services
            .normalize_root_results
            .borrow_mut()
            .push_back(Ok("/Projects".to_string()));
        services
            .config_load_results
            .borrow_mut()
            .push_back(Ok((Config::default(), false)));
        services
            .config_save_results
            .borrow_mut()
            .push_back(Ok(PathBuf::from("/cfg/forge/config.json")));
        services
            .list_dirs_results
            .borrow_mut()
            .push_back(Ok(vec![Dir {
                name: "demo".to_string(),
                path: "/Projects/demo".to_string(),
            }]));
        services.select_dir_results.borrow_mut().push_back(Ok((
            Dir {
                name: "demo".to_string(),
                path: "/Projects/demo".to_string(),
            },
            false,
        )));
        services.select_command_results.borrow_mut().push_back(Ok((
            CommandConfig {
                name: "Codex".to_string(),
                command: "codex".to_string(),
                args: vec!["--fast".to_string()],
            },
            false,
        )));
        services
            .config_load_results
            .borrow_mut()
            .push_back(Ok((Config::default(), false)));

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let exit_code = run_with_services(
            vec![
                OsString::from("forge"),
                OsString::from("--root"),
                OsString::from("~/Projects"),
            ],
            &mut stdout,
            &mut stderr,
            &services,
        );

        assert_eq!(exit_code, EXIT_OK);
        assert_eq!(
            &*services.run_command_calls.borrow(),
            &vec![(
                "codex".to_string(),
                vec!["--fast".to_string()],
                "/Projects/demo".to_string(),
            )]
        );
        assert!(
            String::from_utf8(stderr)
                .unwrap()
                .contains("已保存默认根目录")
        );
    }

    #[test]
    fn test_interactive_first_use_requires_terminal() {
        let mut services = FakeServices::new();
        services.is_terminal = false;
        services
            .config_load_results
            .borrow_mut()
            .push_back(Ok((Config::default(), false)));

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let exit_code = run_with_services(
            vec![OsString::from("forge")],
            &mut stdout,
            &mut stderr,
            &services,
        );

        assert_eq!(exit_code, EXIT_ERROR);
        assert!(
            String::from_utf8(stderr)
                .unwrap()
                .contains("当前环境非交互式终端")
        );
    }

    #[test]
    fn test_config_show_prints_defaults_when_root_missing() {
        let services = FakeServices::new();
        services
            .config_load_results
            .borrow_mut()
            .push_back(Ok((Config::default(), false)));
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let exit_code = run_with_services(
            vec![OsString::from("forge"), OsString::from("config")],
            &mut stdout,
            &mut stderr,
            &services,
        );

        assert_eq!(exit_code, EXIT_OK);
        let output = String::from_utf8(stdout).unwrap();
        assert!(output.contains("默认 root: 未设置"));
        assert!(output.contains("启动命令（内置默认）"));
    }

    #[test]
    fn test_select_command_falls_back_to_default_commands_when_config_load_fails() {
        let services = FakeServices::new();
        services.config_load_results.borrow_mut().push_back(Ok((
            Config {
                root: "/Projects".to_string(),
                ..Config::default()
            },
            true,
        )));
        services
            .select_workspace_results
            .borrow_mut()
            .push_back(Ok((
                WorkspaceResult {
                    choice: WorkspaceChoice::CurrentDir,
                    project_path: "/current".to_string(),
                },
                false,
            )));
        services
            .config_load_results
            .borrow_mut()
            .push_back(Err(anyhow!("broken config")));
        services.select_command_results.borrow_mut().push_back(Ok((
            CommandConfig {
                name: "Claude Code".to_string(),
                command: "claude".to_string(),
                args: Vec::new(),
            },
            false,
        )));

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let exit_code = run_with_services(
            vec![OsString::from("forge")],
            &mut stdout,
            &mut stderr,
            &services,
        );

        assert_eq!(exit_code, EXIT_OK);
        assert_eq!(
            &*services.run_command_calls.borrow(),
            &vec![("claude".to_string(), Vec::new(), "/current".to_string())]
        );
        let err_output = String::from_utf8(stderr).unwrap();
        assert!(err_output.contains("将使用内置默认命令"));
    }
}
