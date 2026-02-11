package runner

import (
	"errors"
	"io"
	"strings"
	"testing"
)

func TestCommandForMode(t *testing.T) {
	cmd, args, err := CommandForMode(ModeClaude)
	if err != nil {
		t.Fatalf("CommandForMode 返回错误：%v", err)
	}
	if cmd != "claude" {
		t.Fatalf("cmd=%q，期望=claude", cmd)
	}
	if len(args) != 1 || args[0] != "--dangerously-skip-permissions" {
		t.Fatalf("args=%v 不符合预期", args)
	}
}

func TestRunCommandWithDeps_LookPathError(t *testing.T) {
	d := deps{
		goos: "darwin",
		chdir: func(string) error {
			return nil
		},
		lookPath: func(string) (string, error) {
			return "", errors.New("not found")
		},
		environ: func() []string {
			return []string{"A=B"}
		},
		exec: func(string, []string, []string) error {
			t.Fatalf("不应调用 exec")
			return nil
		},
		runCmd: func(string, []string, []string, io.Reader, io.Writer, io.Writer) error {
			t.Fatalf("不应调用 runCmd")
			return nil
		},
	}

	err := runCommandWithDeps("codex", []string{"--dangerously-bypass-approvals-and-sandbox"}, "/tmp", d)
	if err == nil {
		t.Fatalf("期望返回错误，但实际为 nil")
	}
	if !strings.Contains(err.Error(), "未找到命令") {
		t.Fatalf("错误信息不包含预期提示：%v", err)
	}
}

func TestRunCommandWithDeps_UnixExecArgs(t *testing.T) {
	var (
		gotPath string
		gotArgv []string
		gotEnv  []string
		gotDir  string
	)

	d := deps{
		goos: "darwin",
		chdir: func(dir string) error {
			gotDir = dir
			return nil
		},
		lookPath: func(string) (string, error) {
			return "/bin/codex", nil
		},
		environ: func() []string {
			return []string{"A=B"}
		},
		exec: func(path string, argv []string, env []string) error {
			gotPath = path
			gotArgv = append([]string(nil), argv...)
			gotEnv = append([]string(nil), env...)
			return nil
		},
		runCmd: func(string, []string, []string, io.Reader, io.Writer, io.Writer) error {
			t.Fatalf("不应调用 runCmd")
			return nil
		},
	}

	err := runCommandWithDeps("codex", []string{"--model", "gpt-5", "--fast"}, "/work", d)
	if err != nil {
		t.Fatalf("runCommandWithDeps 返回错误：%v", err)
	}
	if gotDir != "/work" {
		t.Fatalf("chdir=%q，期望=/work", gotDir)
	}
	if gotPath != "/bin/codex" {
		t.Fatalf("path=%q，期望=/bin/codex", gotPath)
	}
	if len(gotArgv) < 1 || gotArgv[0] != "codex" {
		t.Fatalf("argv[0]=%v 不符合预期：%v", gotArgv, gotArgv)
	}
	if gotEnv[0] != "A=B" {
		t.Fatalf("env=%v 不符合预期", gotEnv)
	}
}

func TestRunCommandWithDeps_WindowsFallback(t *testing.T) {
	var (
		calledRun bool
		gotPath   string
		gotArgs   []string
	)

	d := deps{
		goos: "windows",
		chdir: func(string) error {
			return nil
		},
		lookPath: func(string) (string, error) {
			return "C:\\\\codex.exe", nil
		},
		environ: func() []string {
			return []string{"A=B"}
		},
		exec: func(string, []string, []string) error {
			t.Fatalf("不应调用 exec")
			return nil
		},
		runCmd: func(path string, args []string, env []string, stdin io.Reader, stdout io.Writer, stderr io.Writer) error {
			calledRun = true
			gotPath = path
			gotArgs = append([]string(nil), args...)
			return nil
		},
	}

	err := runCommandWithDeps("codex", []string{"--dangerously-bypass-approvals-and-sandbox"}, "C:\\\\work", d)
	if err != nil {
		t.Fatalf("runCommandWithDeps 返回错误：%v", err)
	}
	if !calledRun {
		t.Fatalf("期望调用 runCmd，但未调用")
	}
	if gotPath != "C:\\\\codex.exe" {
		t.Fatalf("path=%q 不符合预期", gotPath)
	}
	if len(gotArgs) != 1 || gotArgs[0] != "--dangerously-bypass-approvals-and-sandbox" {
		t.Fatalf("args=%v 不符合预期", gotArgs)
	}
}

func TestRunCommandWithDeps_CustomCommand(t *testing.T) {
	var (
		gotPath string
		gotArgv []string
	)

	d := deps{
		goos: "darwin",
		chdir: func(string) error {
			return nil
		},
		lookPath: func(name string) (string, error) {
			return "/usr/local/bin/" + name, nil
		},
		environ: func() []string {
			return []string{"HOME=/home/user"}
		},
		exec: func(path string, argv []string, env []string) error {
			gotPath = path
			gotArgv = append([]string(nil), argv...)
			return nil
		},
		runCmd: func(string, []string, []string, io.Reader, io.Writer, io.Writer) error {
			t.Fatalf("不应调用 runCmd")
			return nil
		},
	}

	err := runCommandWithDeps("aider", []string{"--auto-commits"}, "/projects/myapp", d)
	if err != nil {
		t.Fatalf("runCommandWithDeps 返回错误：%v", err)
	}
	if gotPath != "/usr/local/bin/aider" {
		t.Fatalf("path=%q，期望=/usr/local/bin/aider", gotPath)
	}
	if len(gotArgv) != 2 || gotArgv[0] != "aider" || gotArgv[1] != "--auto-commits" {
		t.Fatalf("argv=%v 不符合预期", gotArgv)
	}
}
