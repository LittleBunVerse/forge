// Package scan 负责扫描根目录的一级子文件夹，并应用过滤与排序规则生成候选列表。
package scan

import (
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
)

type Dir struct {
	Name string
	Path string
}

type Options struct {
	IgnoreNames map[string]struct{}
}

func DefaultIgnoreNames() map[string]struct{} {
	return map[string]struct{}{
		"node_modules": {},
		"target":       {},
		"dist":         {},
		"vendor":       {},
	}
}

func ListDirs(root string, options Options) ([]Dir, error) {
	if strings.TrimSpace(root) == "" {
		return nil, fmt.Errorf("root 不能为空")
	}

	entries, err := os.ReadDir(root)
	if err != nil {
		return nil, fmt.Errorf("读取目录失败：%w", err)
	}

	ignore := options.IgnoreNames
	if ignore == nil {
		ignore = DefaultIgnoreNames()
	}

	dirs := make([]Dir, 0, len(entries))
	for _, entry := range entries {
		name := entry.Name()

		// 强制忽略点目录（例如 .git/.idea/.vscode 等）
		if strings.HasPrefix(name, ".") {
			continue
		}
		if _, ok := ignore[name]; ok {
			continue
		}

		isDir := entry.IsDir()
		if !isDir && entry.Type()&os.ModeSymlink != 0 {
			// 兼容“符号链接指向目录”的情况
			info, infoErr := entry.Info()
			if infoErr == nil && info.IsDir() {
				isDir = true
			}
		}
		if !isDir {
			continue
		}

		dirs = append(dirs, Dir{
			Name: name,
			Path: filepath.Join(root, name),
		})
	}

	sort.Slice(dirs, func(i, j int) bool {
		return dirs[i].Name < dirs[j].Name
	})

	return dirs, nil
}
