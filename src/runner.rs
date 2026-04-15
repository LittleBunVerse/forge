use std::io::{Read, Write};

use anyhow::Result;

type LookPathFn<'a> = dyn Fn(&str) -> Result<String> + 'a;
type ChdirFn<'a> = dyn Fn(&str) -> Result<()> + 'a;
type EnvironFn<'a> = dyn Fn() -> Vec<String> + 'a;
type ExecFn<'a> = dyn Fn(&str, &[String], &[String]) -> Result<()> + 'a;
type RunCmdFn<'a> = dyn Fn(&str, &[String], &[String], &mut dyn Read, &mut dyn Write, &mut dyn Write) -> Result<()>
    + 'a;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Mode {
    Claude,
    Codex,
}

impl Mode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Claude => "Claude Code（claude --dangerously-skip-permissions）",
            Self::Codex => "Codex（codex --dangerously-bypass-approvals-and-sandbox）",
        }
    }
}

pub fn command_for_mode(mode: Mode) -> Result<(String, Vec<String>)> {
    match mode {
        Mode::Claude => Ok((
            "claude".to_string(),
            vec!["--dangerously-skip-permissions".to_string()],
        )),
        Mode::Codex => Ok((
            "codex".to_string(),
            vec!["--dangerously-bypass-approvals-and-sandbox".to_string()],
        )),
    }
}

pub struct Deps<'a> {
    pub goos: &'a str,
    pub look_path: &'a LookPathFn<'a>,
    pub chdir: &'a ChdirFn<'a>,
    pub environ: &'a EnvironFn<'a>,
    pub exec: &'a ExecFn<'a>,
    pub run_cmd: &'a RunCmdFn<'a>,
}

pub fn run_command(_cmd_name: &str, _cmd_args: &[String], _work_dir: &str) -> Result<()> {
    run_command_with_deps(_cmd_name, _cmd_args, _work_dir, default_deps())
}

pub fn run_command_with_deps(
    cmd_name: &str,
    cmd_args: &[String],
    work_dir: &str,
    deps: Deps<'_>,
) -> Result<()> {
    (deps.chdir)(work_dir)?;
    let cmd_path = (deps.look_path)(cmd_name).map_err(|err| {
        anyhow::anyhow!("未找到命令 {cmd_name:?}，请确认已安装且在 PATH 中：{err}")
    })?;

    if deps.goos != "windows" {
        let mut argv = vec![cmd_name.to_string()];
        argv.extend_from_slice(cmd_args);
        return (deps.exec)(&cmd_path, &argv, &(deps.environ)());
    }

    let mut stdin = std::io::empty();
    let mut stdout = std::io::sink();
    let mut stderr = std::io::sink();
    (deps.run_cmd)(
        &cmd_path,
        cmd_args,
        &(deps.environ)(),
        &mut stdin,
        &mut stdout,
        &mut stderr,
    )
}

pub fn run(mode: Mode, work_dir: &str) -> Result<()> {
    let (cmd_name, cmd_args) = command_for_mode(mode)?;
    run_command(&cmd_name, &cmd_args, work_dir)
}

fn default_deps() -> Deps<'static> {
    Deps {
        goos: std::env::consts::OS,
        look_path: &|name| {
            which::which(name)
                .map(|path| path.to_string_lossy().to_string())
                .map_err(|err| anyhow::anyhow!("{err}"))
        },
        chdir: &|dir| {
            std::env::set_current_dir(dir).map_err(|err| anyhow::anyhow!("切换目录失败：{err}"))
        },
        environ: &|| {
            std::env::vars()
                .map(|(key, value)| format!("{key}={value}"))
                .collect()
        },
        exec: &replace_process,
        run_cmd: &run_subprocess,
    }
}

#[cfg(unix)]
fn replace_process(path: &str, argv: &[String], env: &[String]) -> Result<()> {
    use std::os::unix::process::CommandExt;

    let err = std::process::Command::new(path)
        .args(&argv[1..])
        .env_clear()
        .envs(parse_env(env))
        .exec();
    Err(anyhow::anyhow!("{err}"))
}

#[cfg(windows)]
fn replace_process(_path: &str, _argv: &[String], _env: &[String]) -> Result<()> {
    Err(anyhow::anyhow!(
        "Windows 不支持 Replace Process（syscall.Exec），请改用子进程模式"
    ))
}

fn run_subprocess(
    path: &str,
    args: &[String],
    env: &[String],
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<()> {
    let mut command = std::process::Command::new(path);
    command.args(args);
    command.env_clear().envs(parse_env(env));
    command.stdin(std::process::Stdio::inherit());
    command.stdout(std::process::Stdio::inherit());
    command.stderr(std::process::Stdio::inherit());

    let _ = stdin;
    let _ = stdout;
    let _ = stderr;

    command.status().map_err(|err| anyhow::anyhow!("{err}"))?;
    Ok(())
}

fn parse_env(env: &[String]) -> Vec<(String, String)> {
    env.iter()
        .filter_map(|pair| {
            let (key, value) = pair.split_once('=')?;
            Some((key.to_string(), value.to_string()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use anyhow::{Result, anyhow};

    use super::{Deps, Mode, command_for_mode, run_command_with_deps};

    #[test]
    fn test_command_for_mode() {
        let (cmd, args) = command_for_mode(Mode::Claude).expect("CommandForMode 返回错误");
        assert_eq!(cmd, "claude");
        assert_eq!(args, vec!["--dangerously-skip-permissions"]);
    }

    #[test]
    fn test_run_command_with_deps_look_path_error() {
        let chdir = |_dir: &str| -> Result<()> { Ok(()) };
        let look_path = |_name: &str| -> Result<String> { Err(anyhow!("not found")) };
        let environ = || vec!["A=B".to_string()];
        let exec = |_path: &str, _argv: &[String], _env: &[String]| -> Result<()> {
            panic!("不应调用 exec");
        };
        let run_cmd = |_path: &str,
                       _args: &[String],
                       _env: &[String],
                       _stdin: &mut dyn std::io::Read,
                       _stdout: &mut dyn std::io::Write,
                       _stderr: &mut dyn std::io::Write|
         -> Result<()> {
            panic!("不应调用 run_cmd");
        };

        let err = run_command_with_deps(
            "codex",
            &["--dangerously-bypass-approvals-and-sandbox".to_string()],
            "/tmp",
            Deps {
                goos: "darwin",
                look_path: &look_path,
                chdir: &chdir,
                environ: &environ,
                exec: &exec,
                run_cmd: &run_cmd,
            },
        )
        .expect_err("期望返回错误");

        assert!(err.to_string().contains("未找到命令"));
    }

    #[test]
    fn test_run_command_with_deps_unix_exec_args() {
        let got_dir = RefCell::new(String::new());
        let got_path = RefCell::new(String::new());
        let got_argv = RefCell::new(Vec::<String>::new());
        let got_env = RefCell::new(Vec::<String>::new());

        let chdir = |dir: &str| -> Result<()> {
            *got_dir.borrow_mut() = dir.to_string();
            Ok(())
        };
        let look_path = |_name: &str| -> Result<String> { Ok("/bin/codex".to_string()) };
        let environ = || vec!["A=B".to_string()];
        let exec = |path: &str, argv: &[String], env: &[String]| -> Result<()> {
            *got_path.borrow_mut() = path.to_string();
            *got_argv.borrow_mut() = argv.to_vec();
            *got_env.borrow_mut() = env.to_vec();
            Ok(())
        };
        let run_cmd = |_path: &str,
                       _args: &[String],
                       _env: &[String],
                       _stdin: &mut dyn std::io::Read,
                       _stdout: &mut dyn std::io::Write,
                       _stderr: &mut dyn std::io::Write|
         -> Result<()> {
            panic!("不应调用 run_cmd");
        };

        run_command_with_deps(
            "codex",
            &[
                "--model".to_string(),
                "gpt-5".to_string(),
                "--fast".to_string(),
            ],
            "/work",
            Deps {
                goos: "darwin",
                look_path: &look_path,
                chdir: &chdir,
                environ: &environ,
                exec: &exec,
                run_cmd: &run_cmd,
            },
        )
        .expect("runCommandWithDeps 返回错误");

        assert_eq!(&*got_dir.borrow(), "/work");
        assert_eq!(&*got_path.borrow(), "/bin/codex");
        assert_eq!(
            &*got_argv.borrow(),
            &vec![
                "codex".to_string(),
                "--model".to_string(),
                "gpt-5".to_string(),
                "--fast".to_string(),
            ]
        );
        assert_eq!(&*got_env.borrow(), &vec!["A=B".to_string()]);
    }

    #[test]
    fn test_run_command_with_deps_windows_fallback() {
        let called_run = RefCell::new(false);
        let got_path = RefCell::new(String::new());
        let got_args = RefCell::new(Vec::<String>::new());

        let chdir = |_dir: &str| -> Result<()> { Ok(()) };
        let look_path = |_name: &str| -> Result<String> { Ok("C:\\codex.exe".to_string()) };
        let environ = || vec!["A=B".to_string()];
        let exec = |_path: &str, _argv: &[String], _env: &[String]| -> Result<()> {
            panic!("不应调用 exec");
        };
        let run_cmd = |path: &str,
                       args: &[String],
                       _env: &[String],
                       _stdin: &mut dyn std::io::Read,
                       _stdout: &mut dyn std::io::Write,
                       _stderr: &mut dyn std::io::Write|
         -> Result<()> {
            *called_run.borrow_mut() = true;
            *got_path.borrow_mut() = path.to_string();
            *got_args.borrow_mut() = args.to_vec();
            Ok(())
        };

        run_command_with_deps(
            "codex",
            &["--dangerously-bypass-approvals-and-sandbox".to_string()],
            "C:\\work",
            Deps {
                goos: "windows",
                look_path: &look_path,
                chdir: &chdir,
                environ: &environ,
                exec: &exec,
                run_cmd: &run_cmd,
            },
        )
        .expect("runCommandWithDeps 返回错误");

        assert!(*called_run.borrow(), "期望调用 run_cmd");
        assert_eq!(&*got_path.borrow(), "C:\\codex.exe");
        assert_eq!(
            &*got_args.borrow(),
            &vec!["--dangerously-bypass-approvals-and-sandbox".to_string()]
        );
    }

    #[test]
    fn test_run_command_with_deps_custom_command() {
        let got_path = RefCell::new(String::new());
        let got_argv = RefCell::new(Vec::<String>::new());

        let chdir = |_dir: &str| -> Result<()> { Ok(()) };
        let look_path = |name: &str| -> Result<String> { Ok(format!("/usr/local/bin/{name}")) };
        let environ = || vec!["HOME=/home/user".to_string()];
        let exec = |path: &str, argv: &[String], _env: &[String]| -> Result<()> {
            *got_path.borrow_mut() = path.to_string();
            *got_argv.borrow_mut() = argv.to_vec();
            Ok(())
        };
        let run_cmd = |_path: &str,
                       _args: &[String],
                       _env: &[String],
                       _stdin: &mut dyn std::io::Read,
                       _stdout: &mut dyn std::io::Write,
                       _stderr: &mut dyn std::io::Write|
         -> Result<()> {
            panic!("不应调用 run_cmd");
        };

        run_command_with_deps(
            "aider",
            &["--auto-commits".to_string()],
            "/projects/myapp",
            Deps {
                goos: "darwin",
                look_path: &look_path,
                chdir: &chdir,
                environ: &environ,
                exec: &exec,
                run_cmd: &run_cmd,
            },
        )
        .expect("runCommandWithDeps 返回错误");

        assert_eq!(&*got_path.borrow(), "/usr/local/bin/aider");
        assert_eq!(
            &*got_argv.borrow(),
            &vec!["aider".to_string(), "--auto-commits".to_string()]
        );
    }
}
