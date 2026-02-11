package app

import (
	"fmt"
	"io"
	"strings"

	"github.com/LittleBunVerse/forge/internal/config"
	"github.com/LittleBunVerse/forge/internal/pathutil"
)

func runConfigCommand(programName string, args []string, stdout io.Writer, stderr io.Writer) int {
	if len(args) == 0 || args[0] == "show" {
		return runConfigShow(programName, stdout, stderr)
	}

	switch args[0] {
	case "path":
		return runConfigPath(stdout, stderr)
	case "set-root":
		return runConfigSetRoot(programName, args[1:], stdout, stderr)
	case "help", "-h", "--help":
		fallthrough
	default:
		printConfigUsage(programName, stderr)
		return exitUsage
	}
}

func runConfigShow(programName string, stdout io.Writer, stderr io.Writer) int {
	path, err := config.ConfigPath()
	if err != nil {
		fmt.Fprintf(stderr, "获取配置路径失败：%v\n", err)
		return exitError
	}

	cfg, exists, err := config.Load()
	if err != nil {
		fmt.Fprintf(stderr, "读取配置失败：%v\n", err)
		return exitError
	}

	fmt.Fprintf(stdout, "配置文件: %s\n", path)
	if !exists || strings.TrimSpace(cfg.Root) == "" {
		fmt.Fprintln(stdout, "默认 root: 未设置")
		fmt.Fprintf(stdout, "设置方法: %s config set-root \"~/Projects\"\n", programName)
	} else {
		fmt.Fprintf(stdout, "默认 root: %s\n", cfg.Root)
	}

	// 显示命令列表
	commands := cfg.GetCommands()
	isCustom := len(cfg.Commands) > 0
	if isCustom {
		fmt.Fprintln(stdout, "\n启动命令（自定义）:")
	} else {
		fmt.Fprintln(stdout, "\n启动命令（内置默认）:")
	}
	for i, cmd := range commands {
		args := strings.Join(cmd.Args, " ")
		fmt.Fprintf(stdout, "  %d. %s → %s %s\n", i+1, cmd.Name, cmd.Command, args)
	}

	return exitOK
}

func runConfigPath(stdout io.Writer, stderr io.Writer) int {
	path, err := config.ConfigPath()
	if err != nil {
		fmt.Fprintf(stderr, "获取配置路径失败：%v\n", err)
		return exitError
	}
	fmt.Fprintln(stdout, path)
	return exitOK
}

func runConfigSetRoot(programName string, args []string, stdout io.Writer, stderr io.Writer) int {
	if len(args) == 0 {
		root, canceled, err := selectAndPersistRoot("请选择默认根目录（只扫描一级子目录）", programName, stderr)
		if err != nil {
			fmt.Fprintf(stderr, "设置 root 失败：%v\n", err)
			return exitError
		}
		if canceled {
			return exitOK
		}
		fmt.Fprintf(stdout, "默认 root 已更新：%s\n", root)
		return exitOK
	}

	normalized, err := pathutil.NormalizeRoot(args[0])
	if err != nil {
		fmt.Fprintf(stderr, "根目录无效：%v\n", err)
		return exitUsage
	}

	path, err := config.SaveRoot(normalized)
	if err != nil {
		fmt.Fprintf(stderr, "写入配置失败：%v\n", err)
		return exitError
	}

	fmt.Fprintf(stdout, "默认 root 已更新：%s\n", normalized)
	fmt.Fprintf(stdout, "配置文件: %s\n", path)
	return exitOK
}

func printConfigUsage(programName string, out io.Writer) {
	fmt.Fprintf(out, "用法: %s config <command>\n", programName)
	fmt.Fprintln(out, "")
	fmt.Fprintln(out, "命令：")
	fmt.Fprintln(out, "  show              显示当前配置（默认）")
	fmt.Fprintln(out, "  path              输出配置文件路径")
	fmt.Fprintln(out, "  set-root [<dir>]  设置默认 root（不带参数则交互式选择）")
	fmt.Fprintln(out, "")
	fmt.Fprintln(out, "示例：")
	fmt.Fprintf(out, "  %s config\n", programName)
	fmt.Fprintf(out, "  %s config set-root \"~/Projects\"\n", programName)
}
