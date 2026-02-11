package pathutil

import (
	"os"
	"path/filepath"
	"testing"
)

func TestExpandTilde(t *testing.T) {
	home, err := os.UserHomeDir()
	if err != nil {
		t.Skipf("无法获取 HOME 目录：%v", err)
	}

	got, err := ExpandTilde("~")
	if err != nil {
		t.Fatalf("ExpandTilde(~) 返回错误：%v", err)
	}
	if got != home {
		t.Fatalf("ExpandTilde(~)=%q，期望=%q", got, home)
	}

	got, err = ExpandTilde("~/abc")
	if err != nil {
		t.Fatalf("ExpandTilde(~/abc) 返回错误：%v", err)
	}
	want := filepath.Join(home, "abc")
	if got != want {
		t.Fatalf("ExpandTilde(~/abc)=%q，期望=%q", got, want)
	}

	got, err = ExpandTilde("~\\abc")
	if err != nil {
		t.Fatalf("ExpandTilde(~\\\\abc) 返回错误：%v", err)
	}
	if got != want {
		t.Fatalf("ExpandTilde(~\\\\abc)=%q，期望=%q", got, want)
	}
}

func TestNormalizeRoot(t *testing.T) {
	tmp := t.TempDir()

	got, err := NormalizeRoot(tmp)
	if err != nil {
		t.Fatalf("NormalizeRoot 返回错误：%v", err)
	}

	want, err := filepath.Abs(tmp)
	if err != nil {
		t.Fatalf("Abs 返回错误：%v", err)
	}
	if got != want {
		t.Fatalf("NormalizeRoot=%q，期望=%q", got, want)
	}
}
