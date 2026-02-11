package app

import (
	"fmt"
	"io"
)

// root 子命令是 config set-root 的便捷别名：
// - forge root            => 等同于 forge config show
// - forge root set <dir>  => 等同于 forge config set-root <dir>
func runRootCommand(programName string, args []string, stdout io.Writer, stderr io.Writer) int {
	if len(args) == 0 {
		return runConfigShow(programName, stdout, stderr)
	}

	switch args[0] {
	case "set":
		return runConfigSetRoot(programName, args[1:], stdout, stderr)
	case "help", "-h", "--help":
		fallthrough
	default:
		printRootUsage(programName, stderr)
		return exitUsage
	}
}

func printRootUsage(programName string, out io.Writer) {
	fmt.Fprintf(out, "用法: %s root <command>\n", programName)
	fmt.Fprintln(out, "")
	fmt.Fprintln(out, "命令：")
	fmt.Fprintln(out, "  (空)              显示当前默认 root")
	fmt.Fprintln(out, "  set [<dir>]       设置默认 root（不带参数则交互式选择）")
	fmt.Fprintln(out, "")
	fmt.Fprintln(out, "示例：")
	fmt.Fprintf(out, "  %s root\n", programName)
	fmt.Fprintf(out, "  %s root set \"~/Projects\"\n", programName)
}
