// Package pathutil 提供路径处理工具：~ 展开、root 归一化校验，以及默认根目录探测。
package pathutil

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
)

// ExpandTilde 将以 "~" 或 "~/"（含 Windows 的 "~\\"）开头的路径展开为用户 HOME 目录。
func ExpandTilde(path string) (string, error) {
	if path == "" {
		return "", nil
	}

	if path == "~" {
		home, err := os.UserHomeDir()
		if err != nil {
			return "", fmt.Errorf("获取用户 HOME 目录失败：%w", err)
		}
		return home, nil
	}

	if strings.HasPrefix(path, "~/") || strings.HasPrefix(path, "~\\") {
		home, err := os.UserHomeDir()
		if err != nil {
			return "", fmt.Errorf("获取用户 HOME 目录失败：%w", err)
		}
		return filepath.Join(home, path[2:]), nil
	}

	return path, nil
}

// NormalizeRoot 负责 root 路径的归一化与校验：
// 1) 展开 ~
// 2) Clean + Abs
// 3) 校验存在且为目录
func NormalizeRoot(root string) (string, error) {
	expanded, err := ExpandTilde(root)
	if err != nil {
		return "", err
	}

	cleaned := filepath.Clean(expanded)
	absPath, err := filepath.Abs(cleaned)
	if err != nil {
		return "", fmt.Errorf("解析绝对路径失败：%w", err)
	}

	info, err := os.Stat(absPath)
	if err != nil {
		return "", fmt.Errorf("根目录不存在：%w", err)
	}
	if !info.IsDir() {
		return "", fmt.Errorf("根路径不是目录：%s", absPath)
	}

	return absPath, nil
}

// DetectDefaultRoot 在未指定 root 时，尝试为 macOS/开发者环境提供“开箱即用”的默认根目录：
// 1) 优先使用 ~/Projects
// 2) 其次使用 ~/IdeaProjects
// 3) 都不存在则回退到当前目录 "."
func DetectDefaultRoot() string {
	home, err := os.UserHomeDir()
	if err != nil || home == "" {
		return "."
	}

	candidates := []string{
		filepath.Join(home, "Projects"),
		filepath.Join(home, "IdeaProjects"),
	}

	for _, p := range candidates {
		info, statErr := os.Stat(p)
		if statErr == nil && info.IsDir() {
			return p
		}
	}

	return "."
}
