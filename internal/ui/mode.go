// Package ui 提供终端交互：目录模糊选择与启动模式选择。
package ui

import (
	"fmt"
	"strings"

	"github.com/LittleBunVerse/forge/internal/config"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

// ── Command 选择项 ──────────────────────────────────────────

type commandItem struct {
	cfg  config.CommandConfig
	desc string // 展示用的描述：command + args
}

func buildCommandItems(commands []config.CommandConfig) []commandItem {
	items := make([]commandItem, len(commands))
	for i, cmd := range commands {
		desc := cmd.Command
		if len(cmd.Args) > 0 {
			desc += " " + strings.Join(cmd.Args, " ")
		}
		items[i] = commandItem{cfg: cmd, desc: desc}
	}
	return items
}

// ── Bubble Tea Model ────────────────────────────────────────

type commandSelectModel struct {
	items    []commandItem
	cursor   int
	selected config.CommandConfig
	canceled bool
	done     bool
	width    int
}

func (m commandSelectModel) Init() tea.Cmd {
	return nil
}

func (m commandSelectModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.KeyMsg:
		switch msg.String() {
		case "up", "k":
			if m.cursor > 0 {
				m.cursor--
			}
		case "down", "j":
			if m.cursor < len(m.items)-1 {
				m.cursor++
			}
		case "enter":
			m.selected = m.items[m.cursor].cfg
			m.done = true
			return m, tea.Quit
		case "esc", "ctrl+c", "q":
			m.canceled = true
			m.done = true
			return m, tea.Quit
		}
	case tea.WindowSizeMsg:
		m.width = msg.Width
	}
	return m, nil
}

func (m commandSelectModel) View() string {
	if m.done {
		return ""
	}

	var b strings.Builder

	title := TitleStyle.Render("选择启动模式")

	b.WriteString(title + "\n\n")

	for i, item := range m.items {
		if i == m.cursor {
			cursor := CursorBar.String()

			name := lipgloss.NewStyle().
				Bold(true).
				Foreground(colorSecondary).
				Render(item.cfg.Name)

			desc := SubtitleStyle.Render(" " + item.desc)

			b.WriteString(cursor + " " + name + desc + "\n")
		} else {
			name := lipgloss.NewStyle().
				Foreground(colorFg).
				Render(item.cfg.Name)

			desc := lipgloss.NewStyle().
				Foreground(colorDim).
				Render(" " + item.desc)

			b.WriteString("  " + name + desc + "\n")
		}
	}

	help := HelpStyle.Render("↑↓ 选择 · Enter 确认 · Esc 取消")

	b.WriteString("\n" + help)

	return DocStyle.Render(b.String())
}

// ── 对外接口 ────────────────────────────────────────────────

// SelectCommand 显示命令选择 TUI，返回用户选中的 CommandConfig。
func SelectCommand(commands []config.CommandConfig) (config.CommandConfig, bool, error) {
	if len(commands) == 0 {
		commands = config.DefaultCommands()
	}

	items := buildCommandItems(commands)
	m := commandSelectModel{
		items: items,
	}

	p := tea.NewProgram(m)
	finalModel, err := p.Run()
	if err != nil {
		return config.CommandConfig{}, false, fmt.Errorf("命令选择失败：%w", err)
	}

	result := finalModel.(commandSelectModel)
	if result.canceled {
		return config.CommandConfig{}, true, nil
	}

	fmt.Println(SelectedStyle.Render("✓ 已选择: " + result.selected.Name))
	fmt.Println()

	return result.selected, false, nil
}
