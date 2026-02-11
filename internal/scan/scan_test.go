package scan

import (
	"os"
	"path/filepath"
	"testing"
)

func TestListDirs_FilterAndSort(t *testing.T) {
	root := t.TempDir()

	mustMkdir(t, filepath.Join(root, "b"))
	mustMkdir(t, filepath.Join(root, "a"))
	mustMkdir(t, filepath.Join(root, ".git"))
	mustMkdir(t, filepath.Join(root, "node_modules"))
	mustWriteFile(t, filepath.Join(root, "file.txt"), []byte("x"))

	got, err := ListDirs(root, Options{IgnoreNames: DefaultIgnoreNames()})
	if err != nil {
		t.Fatalf("ListDirs 返回错误：%v", err)
	}

	if len(got) != 2 {
		t.Fatalf("期望 2 个目录，实际=%d（%v）", len(got), got)
	}
	if got[0].Name != "a" || got[1].Name != "b" {
		t.Fatalf("排序或过滤不正确：%v", got)
	}
}

func TestListDirs_EmptyRoot(t *testing.T) {
	_, err := ListDirs("", Options{IgnoreNames: DefaultIgnoreNames()})
	if err == nil {
		t.Fatalf("期望 root 为空时报错，但实际为 nil")
	}
}

func mustMkdir(t *testing.T, path string) {
	t.Helper()
	if err := os.MkdirAll(path, 0o755); err != nil {
		t.Fatalf("创建目录失败：%v", err)
	}
}

func mustWriteFile(t *testing.T, path string, data []byte) {
	t.Helper()
	if err := os.WriteFile(path, data, 0o644); err != nil {
		t.Fatalf("写入文件失败：%v", err)
	}
}
