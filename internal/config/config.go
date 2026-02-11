// Package config 提供 forge 的本地配置读写（例如默认 root 目录）。
package config

import (
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/LittleBunVerse/forge/internal/pathutil"
)

const (
	currentAppName = "forge"
	legacyAppName  = "aidev"
)

// CommandConfig 描述一个可执行的启动命令。
type CommandConfig struct {
	Name    string   `json:"name"`    // TUI 中显示的名称
	Command string   `json:"command"` // 可执行文件名（需在 $PATH 中）
	Args    []string `json:"args"`    // 传给命令的参数列表
}

// ProjectConfig 描述一个项目书签，用于快速跳转。
type ProjectConfig struct {
	Name string `json:"name"` // 显示名称（如 "my-blog"）
	Path string `json:"path"` // 项目路径（如 "~/Projects/my-blog"）
}

// DefaultCommands 返回内置默认命令列表。
func DefaultCommands() []CommandConfig {
	return []CommandConfig{
		{
			Name:    "Claude Code",
			Command: "claude",
			Args:    []string{""},
		},
		{
			Name:    "Codex",
			Command: "codex",
			Args:    []string{""},
		},
	}
}

type Config struct {
	Root     string          `json:"root"`
	Commands []CommandConfig `json:"commands,omitempty"`
	Projects []ProjectConfig `json:"projects,omitempty"`
}

// SaveRoot 在尽量保留现有配置（例如 Commands）的前提下，仅更新 root 并写入配置文件。
//
// 说明：
// - 如果配置文件不存在：会创建新配置文件（只写入 root）。
// - 如果配置文件损坏/无法解析：会以 root 为准覆盖写入，避免用户被卡住无法修复。
func SaveRoot(root string) (string, error) {
	normalized := strings.TrimSpace(root)
	if normalized == "" {
		return "", fmt.Errorf("root 不能为空")
	}

	cfg, _, err := Load()
	if err != nil {
		cfg = Config{}
	}
	cfg.Root = normalized

	return Save(cfg)
}

// GetCommands 返回配置中的命令列表；如果用户未自定义则返回默认值。
func (c Config) GetCommands() []CommandConfig {
	if len(c.Commands) == 0 {
		return DefaultCommands()
	}
	return c.Commands
}

// GetProjects 返回配置中的项目书签列表。
func (c Config) GetProjects() []ProjectConfig {
	return c.Projects
}

func AppName() string {
	return currentAppName
}

// BaseConfigDir 返回配置根目录（不含应用子目录）。
//
// 优先级：
// 1) FORGE_CONFIG_DIR
// 2) XDG_CONFIG_HOME
// 3) ~/.config
func BaseConfigDir() (string, error) {
	if v := strings.TrimSpace(os.Getenv("FORGE_CONFIG_DIR")); v != "" {
		return normalizeDir(v)
	}
	if v := strings.TrimSpace(os.Getenv("XDG_CONFIG_HOME")); v != "" {
		return normalizeDir(v)
	}

	home, err := os.UserHomeDir()
	if err != nil {
		return "", fmt.Errorf("获取用户 HOME 目录失败：%w", err)
	}
	return filepath.Join(home, ".config"), nil
}

func AppConfigDir() (string, error) {
	base, err := BaseConfigDir()
	if err != nil {
		return "", err
	}
	return filepath.Join(base, AppName()), nil
}

func ConfigPath() (string, error) {
	dir, err := AppConfigDir()
	if err != nil {
		return "", err
	}
	return filepath.Join(dir, "config.json"), nil
}

func legacyConfigPath() (string, error) {
	base, err := legacyBaseConfigDir()
	if err != nil {
		return "", err
	}
	return filepath.Join(base, legacyAppName, "config.json"), nil
}

func legacyBaseConfigDir() (string, error) {
	if v := strings.TrimSpace(os.Getenv("AIDEV_CONFIG_DIR")); v != "" {
		return normalizeDir(v)
	}
	if v := strings.TrimSpace(os.Getenv("XDG_CONFIG_HOME")); v != "" {
		return normalizeDir(v)
	}

	home, err := os.UserHomeDir()
	if err != nil {
		return "", fmt.Errorf("获取用户 HOME 目录失败：%w", err)
	}
	return filepath.Join(home, ".config"), nil
}

// Load 读取配置文件。
// exists=false 表示配置文件不存在（不算错误）。
func Load() (cfg Config, exists bool, err error) {
	path, err := ConfigPath()
	if err != nil {
		return Config{}, false, err
	}

	data, err := os.ReadFile(path)
	if err != nil {
		if errors.Is(err, os.ErrNotExist) {
			return loadFromLegacyConfig()
		}
		return Config{}, false, fmt.Errorf("读取配置文件失败：%w", err)
	}

	if err := json.Unmarshal(data, &cfg); err != nil {
		return Config{}, true, fmt.Errorf("解析配置文件失败：%w", err)
	}

	return cfg, true, nil
}

func loadFromLegacyConfig() (Config, bool, error) {
	legacyPath, err := legacyConfigPath()
	if err != nil {
		return Config{}, false, nil
	}

	data, err := os.ReadFile(legacyPath)
	if err != nil {
		return Config{}, false, nil
	}

	var cfg Config
	if err := json.Unmarshal(data, &cfg); err != nil {
		return Config{}, false, nil
	}
	if strings.TrimSpace(cfg.Root) == "" {
		return Config{}, false, nil
	}

	// 尝试迁移到新配置路径（失败不阻断使用，仍返回 legacy 配置）。
	_, _ = Save(cfg)

	return cfg, true, nil
}

// Save 写入配置文件（会自动创建目录）。
// 如果 Commands 为空，会自动填充默认命令列表，确保用户首次使用时即可看到并编辑。
func Save(cfg Config) (string, error) {
	if strings.TrimSpace(cfg.Root) == "" {
		return "", fmt.Errorf("root 不能为空")
	}

	// 首次保存时自动写入默认命令，方便用户查看和编辑
	if len(cfg.Commands) == 0 {
		cfg.Commands = DefaultCommands()
	}

	path, err := ConfigPath()
	if err != nil {
		return "", err
	}

	dir := filepath.Dir(path)
	if err := os.MkdirAll(dir, 0o755); err != nil {
		return "", fmt.Errorf("创建配置目录失败：%w", err)
	}

	data, err := json.MarshalIndent(cfg, "", "  ")
	if err != nil {
		return "", fmt.Errorf("序列化配置失败：%w", err)
	}
	data = append(data, '\n')

	tmp, err := os.CreateTemp(dir, "config-*.tmp")
	if err != nil {
		return "", fmt.Errorf("创建临时文件失败：%w", err)
	}
	tmpPath := tmp.Name()
	_ = tmp.Chmod(0o600)

	_, writeErr := tmp.Write(data)
	closeErr := tmp.Close()
	if writeErr != nil {
		_ = os.Remove(tmpPath)
		return "", fmt.Errorf("写入临时配置失败：%w", writeErr)
	}
	if closeErr != nil {
		_ = os.Remove(tmpPath)
		return "", fmt.Errorf("关闭临时配置失败：%w", closeErr)
	}

	if err := os.Rename(tmpPath, path); err != nil {
		// 兜底：部分平台 rename 覆盖行为不一致，直接写入目标文件。
		_ = os.Remove(tmpPath)
		if err := os.WriteFile(path, data, 0o600); err != nil {
			return "", fmt.Errorf("写入配置文件失败：%w", err)
		}
	}

	return path, nil
}

func normalizeDir(dir string) (string, error) {
	expanded, err := pathutil.ExpandTilde(dir)
	if err != nil {
		return "", err
	}
	abs, err := filepath.Abs(filepath.Clean(expanded))
	if err != nil {
		return "", fmt.Errorf("解析配置目录失败：%w", err)
	}
	return abs, nil
}
