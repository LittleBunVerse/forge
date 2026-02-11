// Package ui 提供终端交互：目录模糊选择与启动模式选择。
package ui

import "github.com/charmbracelet/lipgloss"

// ── 色板（使用 ANSI 0-15 色号，自动适配用户终端主题）──────────
var (
	colorPrimary   = lipgloss.Color("12") // Bright Blue — 标题
	colorSecondary = lipgloss.Color("13") // Bright Magenta — 选中项
	colorAccent    = lipgloss.Color("10") // Bright Green — 确认色
	colorDim       = lipgloss.Color("8")  // Bright Black (Gray) — 次要文字
	colorFg        = lipgloss.Color("15") // Bright White — 前景色
	colorBg        = lipgloss.Color("0")  // Black — 背景色
	colorCursorBar = lipgloss.Color("14") // Bright Cyan — 指示条
	colorRose      = lipgloss.Color("9")  // Bright Red — 警告色
	colorYellow    = lipgloss.Color("11") // Bright Yellow — 提示色
)

// ── Banner 渐变色板 ─────────────────────────────────────────
var bannerGradient = []lipgloss.Color{
	lipgloss.Color("14"), // Bright Cyan
	lipgloss.Color("14"), // Bright Cyan
	lipgloss.Color("12"), // Bright Blue
	lipgloss.Color("12"), // Bright Blue
	lipgloss.Color("13"), // Bright Magenta
	lipgloss.Color("13"), // Bright Magenta
}

// ── 通用样式 ──────────────────────────────────────────────────
var (
	// TitleStyle 标题样式
	TitleStyle = lipgloss.NewStyle().
			Bold(true).
			Foreground(colorPrimary).
			MarginBottom(1)

	// SubtitleStyle 副标题 / 描述
	SubtitleStyle = lipgloss.NewStyle().
			Foreground(colorDim)

	// ActiveItemStyle 选中项
	ActiveItemStyle = lipgloss.NewStyle().
			Bold(true).
			Foreground(colorSecondary).
			PaddingLeft(1).
			PaddingRight(2)

	// InactiveItemStyle 未选中项
	InactiveItemStyle = lipgloss.NewStyle().
				Foreground(colorFg).
				PaddingLeft(2).
				PaddingRight(2)

	// CursorBar 左侧指示条
	CursorBar = lipgloss.NewStyle().
			Foreground(colorCursorBar).
			SetString("▌")

	// SelectedStyle 已选中确认
	SelectedStyle = lipgloss.NewStyle().
			Foreground(colorAccent).
			Bold(true)

	// HelpStyle 底部帮助文字
	HelpStyle = lipgloss.NewStyle().
			Foreground(colorDim).
			MarginTop(1)

	// DocStyle 最外层的容器
	DocStyle = lipgloss.NewStyle().
			Margin(1, 2)

	// InputPromptStyle 输入框提示
	InputPromptStyle = lipgloss.NewStyle().
				Foreground(colorSecondary).
				Bold(true)

	// InputTextStyle 输入框文字
	InputTextStyle = lipgloss.NewStyle().
			Foreground(colorFg)

	// BannerStyle Banner 样式
	BannerStyle = lipgloss.NewStyle().
			Foreground(colorPrimary).
			Bold(true)

	// BannerInfoStyle Banner 信息行
	BannerInfoStyle = lipgloss.NewStyle().
			Foreground(colorDim).
			Italic(true)
)

// ── 面板 & 增强样式 ─────────────────────────────────────────
var (
	// PanelStyle 选择面板容器：圆角边框 + 内边距
	PanelStyle = lipgloss.NewStyle().
			Border(lipgloss.RoundedBorder()).
			BorderForeground(colorDim).
			Padding(1, 2)

	// PanelTitleStyle 面板内标题
	PanelTitleStyle = lipgloss.NewStyle().
			Bold(true).
			Foreground(colorPrimary).
			PaddingBottom(1)

	// KeyBadgeStyle 快捷键 badge，例如 [Enter]
	KeyBadgeStyle = lipgloss.NewStyle().
			Foreground(colorCursorBar).
			Bold(true)

	// KeyDescStyle badge 旁的描述文字
	KeyDescStyle = lipgloss.NewStyle().
			Foreground(colorDim)

	// StepStyle 步骤指示器样式
	StepStyle = lipgloss.NewStyle().
			Foreground(colorDim).
			Bold(true)

	// DividerStyle 水平分割线
	DividerStyle = lipgloss.NewStyle().
			Foreground(colorDim)

	// ActiveRowStyle 选中行的背景条纹
	ActiveRowStyle = lipgloss.NewStyle().
			Bold(true).
			Foreground(colorSecondary)

	// CmdDescStyle 命令描述（等宽灰色）
	CmdDescStyle = lipgloss.NewStyle().
			Foreground(colorDim).
			Italic(true)
)

// ── 辅助函数 ────────────────────────────────────────────────

// FormatKeyHelp 格式化快捷键帮助，例如 FormatKeyHelp("↑↓", "选择") => "[↑↓] 选择"
func FormatKeyHelp(key, desc string) string {
	return KeyBadgeStyle.Render("["+key+"]") + " " + KeyDescStyle.Render(desc)
}

// FormatHelpLine 格式化一组快捷键帮助，用分隔符连接
func FormatHelpLine(pairs ...string) string {
	if len(pairs)%2 != 0 {
		return ""
	}
	parts := make([]string, 0, len(pairs)/2)
	for i := 0; i < len(pairs); i += 2 {
		parts = append(parts, FormatKeyHelp(pairs[i], pairs[i+1]))
	}
	result := ""
	for i, p := range parts {
		if i > 0 {
			result += "  "
		}
		result += p
	}
	return result
}

// Divider 返回指定宽度的水平分割线
func Divider(width int) string {
	line := ""
	for i := 0; i < width; i++ {
		line += "─"
	}
	return DividerStyle.Render(line)
}
