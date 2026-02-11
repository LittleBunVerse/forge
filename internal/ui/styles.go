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
