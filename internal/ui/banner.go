package ui

import (
	"fmt"
	"io"
)

const banner = `
  _______ ____  _____   _____ ______ 
 |  _____/ __ \|  __ \ / ____|  ____|
 | |__  | |  | | |__) | |  __| |__   
 |  __| | |  | |  _  /| | |_ |  __|  
 | |    | |__| | | \ \| |__| | |____ 
 |_|     \____/|_|  \_\\_____|______|
`

// PrintBanner 在启动时输出 ASCII Art Logo。
func PrintBanner(w io.Writer) {
	fmt.Fprint(w, BannerStyle.Render(banner))
	fmt.Fprintln(w, BannerInfoStyle.Render("  Forge — 快速启动你的 AI 编程助手"))
	fmt.Fprintln(w)
}
