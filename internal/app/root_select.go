package app

import (
	"fmt"
	"io"
	"os"
	"path/filepath"

	"github.com/LittleBunVerse/forge/internal/config"
	"github.com/LittleBunVerse/forge/internal/pathutil"
	"github.com/LittleBunVerse/forge/internal/ui"
)

func selectAndPersistRoot(label string, programName string, stderr io.Writer) (string, bool, error) {
	if !isTerminal(os.Stdin) {
		return "", false, fmt.Errorf("当前环境非交互式终端，请通过 %s --root 或 %s config set-root 设置根目录", programName, programName)
	}

	options, defaultInput := buildRootOptions()

	for {
		selected, canceled, err := ui.SelectRoot(label, options, defaultInput)
		if err != nil {
			return "", false, err
		}
		if canceled {
			return "", true, nil
		}

		normalized, err := pathutil.NormalizeRoot(selected)
		if err != nil {
			fmt.Fprintf(stderr, "根目录无效：%v\n", err)
			continue
		}

		path, err := config.SaveRoot(normalized)
		if err != nil {
			fmt.Fprintf(stderr, "保存默认根目录失败：%v\n", err)
		} else {
			fmt.Fprintf(stderr, "已保存默认根目录：%s（%s）\n", normalized, path)
		}

		return normalized, false, nil
	}
}

func buildRootOptions() (options []ui.RootOption, defaultInput string) {
	defaultRoot := pathutil.DetectDefaultRoot()
	if defaultRoot != "." {
		defaultInput = defaultRoot
	} else {
		defaultInput = "~/Projects"
	}

	seen := map[string]struct{}{}
	add := func(label, value string) {
		if value == "" {
			return
		}
		if _, ok := seen[value]; ok {
			return
		}
		seen[value] = struct{}{}
		options = append(options, ui.RootOption{Label: label, Value: value})
	}

	if defaultRoot != "." {
		add("推荐："+defaultRoot, defaultRoot)
	}

	if home, err := os.UserHomeDir(); err == nil && home != "" {
		projects := filepath.Join(home, "Projects")
		if dirExists(projects) {
			add("~/Projects", projects)
		}
		ideaProjects := filepath.Join(home, "IdeaProjects")
		if dirExists(ideaProjects) {
			add("~/IdeaProjects", ideaProjects)
		}
	}

	add("当前目录（.）", ".")
	options = append(options, ui.RootOption{Label: "手动输入路径...", IsManual: true})

	return options, defaultInput
}

func dirExists(path string) bool {
	info, err := os.Stat(path)
	return err == nil && info.IsDir()
}

func isTerminal(file *os.File) bool {
	info, err := file.Stat()
	if err != nil {
		return false
	}
	return info.Mode()&os.ModeCharDevice != 0
}
