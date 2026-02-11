package ui

import (
	"fmt"
	"io"
	"strings"

	"github.com/charmbracelet/lipgloss"
)

const bannerArt = `  _______ ____  _____   _____ ______ 
 |  _____/ __ \|  __ \ / ____|  ____|
 | |__  | |  | | |__) | |  __| |__   
 |  __| | |  | |  _  /| | |_ |  __|  
 | |    | |__| | | \ \| |__| | |____ 
 |_|     \____/|_|  \_\_____|______|`

// PrintBanner 在启动时输出带渐变色的 ASCII Art Logo。
// version 为版本号，传空字符串则不显示。
func PrintBanner(w io.Writer, version string) {
	lines := strings.Split(bannerArt, "\n")

	for i, line := range lines {
		colorIndex := i
		if colorIndex >= len(bannerGradient) {
			colorIndex = len(bannerGradient) - 1
		}
		style := lipgloss.NewStyle().
			Foreground(bannerGradient[colorIndex]).
			Bold(true)
		fmt.Fprintln(w, style.Render(line))
	}

	// 信息行
	info := "  Forge — 快速启动你的 AI 编程助手"
	if version != "" && version != "dev" {
		info += "  " + KeyBadgeStyle.Render("v"+version)
	}
	fmt.Fprintln(w, BannerInfoStyle.Render(info))

	// 分割线
	fmt.Fprintln(w, Divider(42))
	fmt.Fprintln(w)
}
