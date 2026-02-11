// Package app 提供 CLI 的主流程编排：解析参数、扫描目录、交互选择，并接管启动 codex/claude。
package app

import (
	"errors"
	"flag"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strings"

	"github.com/LittleBunVerse/forge/internal/config"
	"github.com/LittleBunVerse/forge/internal/pathutil"
	"github.com/LittleBunVerse/forge/internal/runner"
	"github.com/LittleBunVerse/forge/internal/scan"
	"github.com/LittleBunVerse/forge/internal/ui"
)

const (
	exitOK    = 0
	exitError = 1
	exitUsage = 2
)

func Run(programPath string, args []string, stdout io.Writer, stderr io.Writer) int {
	programName := filepath.Base(programPath)

	if len(args) > 0 {
		switch args[0] {
		case "config", "cfg":
			return runConfigCommand(programName, args[1:], stdout, stderr)
		case "root":
			return runRootCommand(programName, args[1:], stdout, stderr)
		case "version", "-v", "--version":
			return runVersionCommand(programName, stdout)
		}
	}

	ui.PrintBanner(stderr)

	flagSet := flag.NewFlagSet(programName, flag.ContinueOnError)
	flagSet.SetOutput(stderr)

	var rootFlag string
	var versionFlag bool
	flagSet.StringVar(&rootFlag, "root", "", "扫描根目录（可选；未指定时读取本地配置；首次使用会引导选择并持久化）")
	flagSet.BoolVar(&versionFlag, "version", false, "输出版本信息并退出")
	flagSet.BoolVar(&versionFlag, "v", false, "输出版本信息并退出（简写）")

	flagSet.Usage = func() {
		fmt.Fprintf(stderr, "用法: %s [--root <dir>] [<dir>]\n", programName)
		fmt.Fprintln(stderr, "")
		fmt.Fprintln(stderr, "参数：")
		flagSet.PrintDefaults()
		fmt.Fprintln(stderr, "")
		fmt.Fprintln(stderr, "说明：")
		fmt.Fprintln(stderr, "  - 仅扫描指定根目录的一级子文件夹（Depth=1）")
		fmt.Fprintln(stderr, "  - 强制忽略以 . 开头的目录（如 .git/.idea/.vscode）")
		fmt.Fprintln(stderr, "  - 额外忽略：node_modules/target/dist/vendor")
		fmt.Fprintln(stderr, "  - 默认 root 会持久化到本地配置文件（可通过子命令修改）：")
		fmt.Fprintf(stderr, "      %s config set-root \"~/Projects\"\n", programName)
		fmt.Fprintln(stderr, "")
		fmt.Fprintln(stderr, "示例：")
		fmt.Fprintf(stderr, "  %s --root \"~/Projects\"\n", programName)
		fmt.Fprintf(stderr, "  %s \"~/IdeaProjects\"\n", programName)
	}

	if err := flagSet.Parse(args); err != nil {
		if errors.Is(err, flag.ErrHelp) {
			return exitOK
		}
		fmt.Fprintf(stderr, "参数解析失败：%v\n", err)
		return exitUsage
	}

	if versionFlag {
		return runVersionCommand(programName, stdout)
	}

	// 如果用户通过 --root 或位置参数显式指定了 root，走原有的"扫描子文件夹"流程。
	explicitRoot := strings.TrimSpace(rootFlag)
	if explicitRoot == "" && flagSet.NArg() > 0 {
		explicitRoot = flagSet.Arg(0)
	}

	if strings.TrimSpace(explicitRoot) != "" {
		return runWithExplicitRoot(explicitRoot, programName, stdout, stderr)
	}

	// 无显式 root：走"统一工作区选择"流程。
	return runInteractiveWorkspace(programName, stdout, stderr)
}

// ── 显式指定 root 的流程（保持原有逻辑）─────────────────────

func runWithExplicitRoot(root, programName string, stdout, stderr io.Writer) int {
	normalizedRoot, err := pathutil.NormalizeRoot(root)
	if err != nil {
		fmt.Fprintf(stderr, "根目录无效：%v\n", err)
		return exitUsage
	}

	// 首次使用且用户显式指定 root 时：自动将其保存为默认 root。
	if strings.TrimSpace(os.Getenv("FORGE_ROOT")) == "" && strings.TrimSpace(os.Getenv("AIDEV_ROOT")) == "" {
		_, exists, loadErr := config.Load()
		if loadErr != nil {
			fmt.Fprintf(stderr, "读取本地配置失败（将尝试覆盖写入）：%v\n", loadErr)
		}
		if !exists || loadErr != nil {
			if path, saveErr := config.Save(config.Config{Root: normalizedRoot}); saveErr != nil {
				fmt.Fprintf(stderr, "保存默认根目录失败：%v\n", saveErr)
			} else {
				fmt.Fprintf(stderr, "已保存默认根目录：%s（%s）\n", normalizedRoot, path)
			}
		}
	}

	return scanAndSelectDir(normalizedRoot, stdout, stderr)
}

// ── 统一工作区选择流程 ──────────────────────────────────────

func runInteractiveWorkspace(programName string, stdout, stderr io.Writer) int {
	// 从环境变量读取临时覆盖的 root
	envRoot := strings.TrimSpace(os.Getenv("FORGE_ROOT"))
	if envRoot == "" {
		envRoot = strings.TrimSpace(os.Getenv("AIDEV_ROOT"))
	}
	if envRoot != "" {
		normalizedRoot, err := pathutil.NormalizeRoot(envRoot)
		if err != nil {
			fmt.Fprintf(stderr, "环境变量根目录无效：%v\n", err)
			return exitError
		}
		return scanAndSelectDir(normalizedRoot, stdout, stderr)
	}

	// 加载配置
	cfg, _, loadErr := config.Load()
	if loadErr != nil {
		fmt.Fprintf(stderr, "读取本地配置失败：%v\n", loadErr)
	}

	savedRoot := strings.TrimSpace(cfg.Root)
	projects := cfg.GetProjects()

	// 如果没有已保存的 root 且没有项目书签 → 首次使用，引导选择 root
	if savedRoot == "" && len(projects) == 0 {
		if !isTerminal(os.Stdin) {
			fmt.Fprintf(stderr, "当前环境非交互式终端，请通过 %s --root 或 %s config set-root 设置根目录\n", programName, programName)
			return exitError
		}
		root, canceled, err := selectAndPersistRoot("首次使用：请选择默认根目录（只扫描一级子目录）", programName, stderr)
		if err != nil {
			fmt.Fprintf(stderr, "选择根目录失败：%v\n", err)
			return exitError
		}
		if canceled {
			return exitOK
		}
		normalizedRoot, err := pathutil.NormalizeRoot(root)
		if err != nil {
			fmt.Fprintf(stderr, "根目录无效：%v\n", err)
			return exitError
		}
		return scanAndSelectDir(normalizedRoot, stdout, stderr)
	}

	// 显示统一工作区选择 TUI
	wsResult, canceled, err := ui.SelectWorkspace(savedRoot, projects)
	if err != nil {
		fmt.Fprintf(stderr, "工作区选择失败：%v\n", err)
		return exitError
	}
	if canceled {
		return exitOK
	}

	var projectDir string

	switch wsResult.Choice {
	case ui.WorkspaceCurrentDir:
		// 直接使用当前目录
		projectDir = wsResult.ProjectPath
		fmt.Fprintln(stderr, ui.SelectedStyle.Render("✓ 已选择: 当前目录"))
		fmt.Fprintln(stderr, ui.SubtitleStyle.Render("  "+projectDir))
		fmt.Fprintln(stderr)

	case ui.WorkspaceProject:
		// 直接使用项目书签路径
		normalized, err := pathutil.NormalizeRoot(wsResult.ProjectPath)
		if err != nil {
			fmt.Fprintf(stderr, "项目路径无效：%v\n", err)
			return exitError
		}
		projectDir = normalized
		fmt.Fprintln(stderr, ui.SelectedStyle.Render("✓ 已选择: "+filepath.Base(projectDir)))
		fmt.Fprintln(stderr, ui.SubtitleStyle.Render("  "+projectDir))
		fmt.Fprintln(stderr)

	case ui.WorkspaceBrowseRoot:
		// 从已保存的根目录浏览子文件夹
		normalizedRoot, err := pathutil.NormalizeRoot(savedRoot)
		if err != nil {
			fmt.Fprintf(stderr, "已保存的根目录无效：%v\n请重新选择根目录…\n", err)
			root, canceled, err := selectAndPersistRoot("请选择项目根目录（只扫描一级子目录）", programName, stderr)
			if err != nil {
				fmt.Fprintf(stderr, "选择根目录失败：%v\n", err)
				return exitError
			}
			if canceled {
				return exitOK
			}
			normalizedRoot, err = pathutil.NormalizeRoot(root)
			if err != nil {
				fmt.Fprintf(stderr, "根目录无效：%v\n", err)
				return exitError
			}
		}
		return scanAndSelectDir(normalizedRoot, stdout, stderr)

	case ui.WorkspaceNewRoot:
		// 选择新的根目录
		if !isTerminal(os.Stdin) {
			fmt.Fprintf(stderr, "当前环境非交互式终端，请通过 %s --root 或 %s config set-root 设置根目录\n", programName, programName)
			return exitError
		}
		root, canceled, err := selectAndPersistRoot("选择新的根目录（只扫描一级子目录）", programName, stderr)
		if err != nil {
			fmt.Fprintf(stderr, "选择根目录失败：%v\n", err)
			return exitError
		}
		if canceled {
			return exitOK
		}
		normalizedRoot, err := pathutil.NormalizeRoot(root)
		if err != nil {
			fmt.Fprintf(stderr, "根目录无效：%v\n", err)
			return exitError
		}
		return scanAndSelectDir(normalizedRoot, stdout, stderr)
	}

	// 到达这里说明已选定 projectDir（CurrentDir 或 Project），直接选择命令并执行。
	return selectCommandAndRun(projectDir, stderr)
}

// ── 公共流程片段 ────────────────────────────────────────────

// scanAndSelectDir 扫描根目录的一级子文件夹，让用户选择后执行命令。
func scanAndSelectDir(normalizedRoot string, stdout io.Writer, stderr io.Writer) int {
	dirs, err := scan.ListDirs(normalizedRoot, scan.Options{
		IgnoreNames: scan.DefaultIgnoreNames(),
	})
	if err != nil {
		fmt.Fprintf(stderr, "扫描目录失败：%v\n", err)
		return exitError
	}
	if len(dirs) == 0 {
		fmt.Fprintf(stderr, "根目录下没有可选项目目录：%s\n", normalizedRoot)
		return exitError
	}

	selectedDir, canceled, err := ui.SelectDir(dirs)
	if err != nil {
		fmt.Fprintf(stderr, "选择目录失败：%v\n", err)
		return exitError
	}
	if canceled {
		return exitOK
	}

	return selectCommandAndRun(selectedDir.Path, stderr)
}

// selectCommandAndRun 选择命令并在指定目录中执行。
func selectCommandAndRun(projectDir string, stderr io.Writer) int {
	cfg2, _, _ := config.Load()
	commands := cfg2.GetCommands()

	selectedCmd, canceled, err := ui.SelectCommand(commands)
	if err != nil {
		fmt.Fprintf(stderr, "选择命令失败：%v\n", err)
		return exitError
	}
	if canceled {
		return exitOK
	}

	if err := runner.RunCommand(selectedCmd.Command, selectedCmd.Args, projectDir); err != nil {
		fmt.Fprintf(stderr, "%v\n", err)
		return exitError
	}

	return exitOK
}
