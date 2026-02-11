// forge 入口：扫描项目目录并接管启动 codex/claude。
package main

import (
	"os"

	"github.com/LittleBunVerse/forge/internal/app"
)

func main() {
	os.Exit(app.Run(os.Args[0], os.Args[1:], os.Stdout, os.Stderr))
}
