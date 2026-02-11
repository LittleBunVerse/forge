package config

import (
	"os"
	"path/filepath"
	"testing"
)

func TestConfigPath_UsesForgeConfigDir(t *testing.T) {
	base := t.TempDir()
	t.Setenv("FORGE_CONFIG_DIR", base)
	// 避免读取到本机真实 legacy 配置影响测试。
	t.Setenv("AIDEV_CONFIG_DIR", base)

	got, err := ConfigPath()
	if err != nil {
		t.Fatalf("ConfigPath 返回错误：%v", err)
	}

	want := filepath.Join(base, AppName(), "config.json")
	if got != want {
		t.Fatalf("ConfigPath=%q，期望=%q", got, want)
	}
}

func TestLoad_Missing(t *testing.T) {
	base := t.TempDir()
	t.Setenv("FORGE_CONFIG_DIR", base)
	// 避免读取到本机真实 legacy 配置影响测试。
	t.Setenv("AIDEV_CONFIG_DIR", base)

	_, exists, err := Load()
	if err != nil {
		t.Fatalf("Load 返回错误：%v", err)
	}
	if exists {
		t.Fatalf("期望 exists=false，但实际为 true")
	}
}

func TestSaveAndLoad(t *testing.T) {
	base := t.TempDir()
	t.Setenv("FORGE_CONFIG_DIR", base)
	// 避免读取到本机真实 legacy 配置影响测试。
	t.Setenv("AIDEV_CONFIG_DIR", base)

	root := t.TempDir()

	path, err := Save(Config{Root: root})
	if err != nil {
		t.Fatalf("Save 返回错误：%v", err)
	}
	if _, err := os.Stat(path); err != nil {
		t.Fatalf("配置文件未写入：%v", err)
	}

	cfg, exists, err := Load()
	if err != nil {
		t.Fatalf("Load 返回错误：%v", err)
	}
	if !exists {
		t.Fatalf("期望 exists=true，但实际为 false")
	}
	if cfg.Root != root {
		t.Fatalf("cfg.Root=%q，期望=%q", cfg.Root, root)
	}
	// Save 应自动填充默认命令
	if len(cfg.Commands) != 2 {
		t.Fatalf("期望 Save 自动写入 2 个默认命令，实际=%d", len(cfg.Commands))
	}
	if cfg.Commands[0].Name != "Claude Code" {
		t.Fatalf("命令名=%q，期望=Claude Code", cfg.Commands[0].Name)
	}
}

func TestGetCommands_DefaultWhenEmpty(t *testing.T) {
	cfg := Config{Root: "/tmp"}
	commands := cfg.GetCommands()

	if len(commands) != 2 {
		t.Fatalf("期望 2 个默认命令，实际=%d", len(commands))
	}
	if commands[0].Name != "Claude Code" {
		t.Fatalf("第一个命令名=%q，期望=Claude Code", commands[0].Name)
	}
	if commands[0].Command != "claude" {
		t.Fatalf("第一个命令=%q，期望=claude", commands[0].Command)
	}
	if commands[1].Name != "Codex" {
		t.Fatalf("第二个命令名=%q，期望=Codex", commands[1].Name)
	}
}

func TestGetCommands_CustomOverride(t *testing.T) {
	cfg := Config{
		Root: "/tmp",
		Commands: []CommandConfig{
			{Name: "Aider", Command: "aider", Args: []string{"--auto-commits"}},
		},
	}
	commands := cfg.GetCommands()

	if len(commands) != 1 {
		t.Fatalf("期望 1 个自定义命令，实际=%d", len(commands))
	}
	if commands[0].Name != "Aider" {
		t.Fatalf("命令名=%q，期望=Aider", commands[0].Name)
	}
	if commands[0].Command != "aider" {
		t.Fatalf("命令=%q，期望=aider", commands[0].Command)
	}
	if len(commands[0].Args) != 1 || commands[0].Args[0] != "--auto-commits" {
		t.Fatalf("args=%v 不符合预期", commands[0].Args)
	}
}

func TestSaveAndLoadWithCommands(t *testing.T) {
	base := t.TempDir()
	t.Setenv("FORGE_CONFIG_DIR", base)
	// 避免读取到本机真实 legacy 配置影响测试。
	t.Setenv("AIDEV_CONFIG_DIR", base)

	root := t.TempDir()
	customCmds := []CommandConfig{
		{Name: "MyTool", Command: "mytool", Args: []string{"--flag1", "--flag2"}},
		{Name: "AnotherTool", Command: "another", Args: nil},
	}

	path, err := Save(Config{Root: root, Commands: customCmds})
	if err != nil {
		t.Fatalf("Save 返回错误：%v", err)
	}
	if _, err := os.Stat(path); err != nil {
		t.Fatalf("配置文件未写入：%v", err)
	}

	cfg, exists, err := Load()
	if err != nil {
		t.Fatalf("Load 返回错误：%v", err)
	}
	if !exists {
		t.Fatalf("期望 exists=true，但实际为 false")
	}
	if len(cfg.Commands) != 2 {
		t.Fatalf("期望 2 个命令，实际=%d", len(cfg.Commands))
	}
	if cfg.Commands[0].Name != "MyTool" {
		t.Fatalf("命令名=%q，期望=MyTool", cfg.Commands[0].Name)
	}
	if cfg.Commands[1].Command != "another" {
		t.Fatalf("命令=%q，期望=another", cfg.Commands[1].Command)
	}
}

func TestSaveRoot_PreservesExistingCommands(t *testing.T) {
	base := t.TempDir()
	t.Setenv("FORGE_CONFIG_DIR", base)
	// 避免读取到本机真实 legacy 配置影响测试。
	t.Setenv("AIDEV_CONFIG_DIR", base)

	root1 := t.TempDir()
	root2 := t.TempDir()
	customCmds := []CommandConfig{
		{Name: "MyTool", Command: "mytool", Args: []string{"--flag1"}},
	}

	if _, err := Save(Config{Root: root1, Commands: customCmds}); err != nil {
		t.Fatalf("初始化配置失败：%v", err)
	}

	if _, err := SaveRoot(root2); err != nil {
		t.Fatalf("SaveRoot 返回错误：%v", err)
	}

	cfg, exists, err := Load()
	if err != nil {
		t.Fatalf("Load 返回错误：%v", err)
	}
	if !exists {
		t.Fatalf("期望 exists=true，但实际为 false")
	}
	if cfg.Root != root2 {
		t.Fatalf("cfg.Root=%q，期望=%q", cfg.Root, root2)
	}
	if len(cfg.Commands) != 1 {
		t.Fatalf("期望 1 个命令，实际=%d", len(cfg.Commands))
	}
	if cfg.Commands[0].Name != "MyTool" {
		t.Fatalf("命令名=%q，期望=MyTool", cfg.Commands[0].Name)
	}
	if cfg.Commands[0].Command != "mytool" {
		t.Fatalf("命令=%q，期望=mytool", cfg.Commands[0].Command)
	}
}

func TestLoad_MigrateFromLegacyAidevConfig(t *testing.T) {
	forgeBase := t.TempDir()
	legacyBase := t.TempDir()
	t.Setenv("FORGE_CONFIG_DIR", forgeBase)
	t.Setenv("AIDEV_CONFIG_DIR", legacyBase)

	root := t.TempDir()

	legacyPath := filepath.Join(legacyBase, legacyAppName, "config.json")
	if err := os.MkdirAll(filepath.Dir(legacyPath), 0o755); err != nil {
		t.Fatalf("创建 legacy 配置目录失败：%v", err)
	}
	if err := os.WriteFile(legacyPath, []byte("{\"root\":\""+root+"\"}\n"), 0o600); err != nil {
		t.Fatalf("写入 legacy 配置失败：%v", err)
	}

	cfg, exists, err := Load()
	if err != nil {
		t.Fatalf("Load 返回错误：%v", err)
	}
	if !exists {
		t.Fatalf("期望 exists=true，但实际为 false")
	}
	if cfg.Root != root {
		t.Fatalf("cfg.Root=%q，期望=%q", cfg.Root, root)
	}

	newPath := filepath.Join(forgeBase, currentAppName, "config.json")
	if _, err := os.Stat(newPath); err != nil {
		t.Fatalf("期望已迁移写入新配置文件，但未找到：%v", err)
	}
}

func TestGetProjects_EmptyWhenNone(t *testing.T) {
	cfg := Config{Root: "/tmp"}
	projects := cfg.GetProjects()

	if len(projects) != 0 {
		t.Fatalf("期望 0 个项目书签，实际=%d", len(projects))
	}
}

func TestSaveAndLoadWithProjects(t *testing.T) {
	base := t.TempDir()
	t.Setenv("FORGE_CONFIG_DIR", base)
	t.Setenv("AIDEV_CONFIG_DIR", base)

	root := t.TempDir()
	customProjects := []ProjectConfig{
		{Name: "my-blog", Path: "~/Projects/my-blog"},
		{Name: "todo-app", Path: "/Users/test/todo-app"},
	}

	path, err := Save(Config{Root: root, Projects: customProjects})
	if err != nil {
		t.Fatalf("Save 返回错误：%v", err)
	}
	if _, err := os.Stat(path); err != nil {
		t.Fatalf("配置文件未写入：%v", err)
	}

	cfg, exists, err := Load()
	if err != nil {
		t.Fatalf("Load 返回错误：%v", err)
	}
	if !exists {
		t.Fatalf("期望 exists=true，但实际为 false")
	}
	if len(cfg.Projects) != 2 {
		t.Fatalf("期望 2 个项目书签，实际=%d", len(cfg.Projects))
	}
	if cfg.Projects[0].Name != "my-blog" {
		t.Fatalf("项目名=%q，期望=my-blog", cfg.Projects[0].Name)
	}
	if cfg.Projects[1].Path != "/Users/test/todo-app" {
		t.Fatalf("项目路径=%q，期望=/Users/test/todo-app", cfg.Projects[1].Path)
	}
}

func TestSaveRoot_PreservesExistingProjects(t *testing.T) {
	base := t.TempDir()
	t.Setenv("FORGE_CONFIG_DIR", base)
	t.Setenv("AIDEV_CONFIG_DIR", base)

	root1 := t.TempDir()
	root2 := t.TempDir()
	customProjects := []ProjectConfig{
		{Name: "my-blog", Path: "~/Projects/my-blog"},
	}

	if _, err := Save(Config{Root: root1, Projects: customProjects}); err != nil {
		t.Fatalf("初始化配置失败：%v", err)
	}

	if _, err := SaveRoot(root2); err != nil {
		t.Fatalf("SaveRoot 返回错误：%v", err)
	}

	cfg, exists, err := Load()
	if err != nil {
		t.Fatalf("Load 返回错误：%v", err)
	}
	if !exists {
		t.Fatalf("期望 exists=true，但实际为 false")
	}
	if cfg.Root != root2 {
		t.Fatalf("cfg.Root=%q，期望=%q", cfg.Root, root2)
	}
	if len(cfg.Projects) != 1 {
		t.Fatalf("期望 1 个项目书签，实际=%d", len(cfg.Projects))
	}
	if cfg.Projects[0].Name != "my-blog" {
		t.Fatalf("项目名=%q，期望=my-blog", cfg.Projects[0].Name)
	}
}
