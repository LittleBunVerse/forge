package app

import (
	"fmt"
	"io"
	"runtime"
)

// 版本信息：默认值用于本地开发，发布时建议通过 -ldflags 注入（例如 GoReleaser）。
var (
	Version = "dev"
	Commit  = "none"
	Date    = "unknown"
)

func runVersionCommand(programName string, stdout io.Writer) int {
	fmt.Fprintf(stdout, "%s %s\n", programName, Version)
	fmt.Fprintf(stdout, "commit:  %s\n", Commit)
	fmt.Fprintf(stdout, "built:   %s\n", Date)
	fmt.Fprintf(stdout, "go:      %s\n", runtime.Version())
	fmt.Fprintf(stdout, "os/arch: %s/%s\n", runtime.GOOS, runtime.GOARCH)
	return exitOK
}
