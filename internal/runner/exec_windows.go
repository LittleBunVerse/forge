//go:build windows

// Windows 下不支持 Replace Process，保留占位实现用于提示并走子进程降级逻辑。
package runner

import "fmt"

func replaceProcess(path string, argv []string, env []string) error {
	return fmt.Errorf("Windows 不支持 Replace Process（syscall.Exec），请改用子进程模式")
}
