//go:build !windows

// Unix 下使用 syscall.Exec 替换当前进程，实现“当前终端接管”的体验。
package runner

import "syscall"

func replaceProcess(path string, argv []string, env []string) error {
	return syscall.Exec(path, argv, env)
}
