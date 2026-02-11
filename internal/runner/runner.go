// Package runner 负责在选中目录内接管执行 codex/claude，并提供可测试的命令构建逻辑。
package runner

import (
	"fmt"
	"io"
	"os"
	"os/exec"
	"runtime"
)

// Mode 保留向后兼容（内置模式枚举）。
type Mode int

const (
	ModeClaude Mode = iota
	ModeCodex
)

func (m Mode) String() string {
	switch m {
	case ModeClaude:
		return "Claude Code（claude --dangerously-skip-permissions）"
	case ModeCodex:
		return "Codex（codex --dangerously-bypass-approvals-and-sandbox）"
	default:
		return "未知模式"
	}
}

// CommandForMode 保留向后兼容。
func CommandForMode(mode Mode) (string, []string, error) {
	switch mode {
	case ModeClaude:
		return "claude", []string{"--dangerously-skip-permissions"}, nil
	case ModeCodex:
		return "codex", []string{"--dangerously-bypass-approvals-and-sandbox"}, nil
	default:
		return "", nil, fmt.Errorf("无效的启动模式：%d", mode)
	}
}

type deps struct {
	goos     string
	lookPath func(string) (string, error)
	chdir    func(string) error
	environ  func() []string
	exec     func(string, []string, []string) error
	runCmd   func(string, []string, []string, io.Reader, io.Writer, io.Writer) error
}

func defaultDeps() deps {
	return deps{
		goos:     runtime.GOOS,
		lookPath: exec.LookPath,
		chdir:    os.Chdir,
		environ:  os.Environ,
		exec:     replaceProcess,
		runCmd:   runCommand,
	}
}

// RunCommand 是新的主入口：直接接受命令名和参数列表。
func RunCommand(cmdName string, cmdArgs []string, workDir string) error {
	return runCommandWithDeps(cmdName, cmdArgs, workDir, defaultDeps())
}

func runCommandWithDeps(cmdName string, cmdArgs []string, workDir string, d deps) error {
	if err := d.chdir(workDir); err != nil {
		return fmt.Errorf("切换目录失败：%w", err)
	}

	cmdPath, err := d.lookPath(cmdName)
	if err != nil {
		return fmt.Errorf("未找到命令 %q，请确认已安装且在 PATH 中：%w", cmdName, err)
	}

	if d.goos != "windows" {
		argv := append([]string{cmdName}, cmdArgs...)
		return d.exec(cmdPath, argv, d.environ())
	}

	// Windows 下无法 Replace Process，降级为启动子进程并透传 stdio。
	return d.runCmd(cmdPath, cmdArgs, d.environ(), os.Stdin, os.Stdout, os.Stderr)
}

// Run 保留向后兼容，内部转调 RunCommand。
func Run(mode Mode, workDir string) error {
	cmdName, cmdArgs, err := CommandForMode(mode)
	if err != nil {
		return err
	}
	return RunCommand(cmdName, cmdArgs, workDir)
}

func runCommand(path string, args []string, env []string, stdin io.Reader, stdout io.Writer, stderr io.Writer) error {
	cmd := exec.Command(path, args...)
	cmd.Env = env
	cmd.Stdin = stdin
	cmd.Stdout = stdout
	cmd.Stderr = stderr
	return cmd.Run()
}
