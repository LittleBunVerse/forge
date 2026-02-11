package ui

import (
	"fmt"
	"strings"

	"github.com/charmbracelet/bubbles/textinput"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

type RootOption struct {
	Label    string
	Value    string
	IsManual bool
}

// ── 阶段状态 ────────────────────────────────────────────────

type rootPhase int

const (
	rootPhaseSelect rootPhase = iota
	rootPhaseInput
)

// ── Bubble Tea Model ────────────────────────────────────────

type rootSelectModel struct {
	phase    rootPhase
	options  []RootOption
	cursor   int
	selected string
	canceled bool
	done     bool

	// 手动输入阶段
	input textinput.Model
}

func newRootSelectModel(options []RootOption, defaultInput string) rootSelectModel {
	ti := textinput.New()
	ti.Placeholder = "请输入根目录路径，例如 ~/Projects"
	ti.Focus()
	ti.CharLimit = 256
	ti.Width = 50
	ti.PromptStyle = InputPromptStyle
	ti.TextStyle = InputTextStyle
	ti.Cursor.Style = lipgloss.NewStyle().Foreground(colorCursorBar)
	if defaultInput != "" {
		ti.SetValue(defaultInput)
	}

	return rootSelectModel{
		phase:   rootPhaseSelect,
		options: options,
		input:   ti,
	}
}

func (m rootSelectModel) Init() tea.Cmd {
	return nil
}

func (m rootSelectModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch m.phase {
	case rootPhaseSelect:
		return m.updateSelect(msg)
	case rootPhaseInput:
		return m.updateInput(msg)
	}
	return m, nil
}

func (m rootSelectModel) updateSelect(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.KeyMsg:
		switch msg.String() {
		case "up", "k":
			if m.cursor > 0 {
				m.cursor--
			}
		case "down", "j":
			if m.cursor < len(m.options)-1 {
				m.cursor++
			}
		case "enter":
			opt := m.options[m.cursor]
			if opt.IsManual {
				m.phase = rootPhaseInput
				return m, textinput.Blink
			}
			m.selected = strings.TrimSpace(opt.Value)
			m.done = true
			return m, tea.Quit
		case "esc", "ctrl+c", "q":
			m.canceled = true
			m.done = true
			return m, tea.Quit
		}
	}
	return m, nil
}

func (m rootSelectModel) updateInput(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.KeyMsg:
		switch msg.String() {
		case "enter":
			value := strings.TrimSpace(m.input.Value())
			if value != "" {
				m.selected = value
				m.done = true
				return m, tea.Quit
			}
		case "esc":
			// 返回选择阶段
			m.phase = rootPhaseSelect
			return m, nil
		case "ctrl+c":
			m.canceled = true
			m.done = true
			return m, tea.Quit
		}
	}

	var cmd tea.Cmd
	m.input, cmd = m.input.Update(msg)
	return m, cmd
}

func (m rootSelectModel) View() string {
	if m.done {
		return ""
	}

	switch m.phase {
	case rootPhaseSelect:
		return m.viewSelect()
	case rootPhaseInput:
		return m.viewInput()
	}
	return ""
}

func (m rootSelectModel) viewSelect() string {
	var b strings.Builder

	title := PanelTitleStyle.Render("选择根目录")
	b.WriteString(title + "\n\n")

	for i, opt := range m.options {
		if i == m.cursor {
			cursor := CursorBar.String()
			arrow := lipgloss.NewStyle().Foreground(colorCursorBar).Bold(true).Render(" ›")

			label := lipgloss.NewStyle().
				Bold(true).
				Foreground(colorSecondary).
				Render(opt.Label)

			line := cursor + " " + label + arrow
			if opt.Value != "" {
				path := SubtitleStyle.Render("  " + opt.Value)
				line += path
			}
			b.WriteString(line + "\n")
		} else {
			label := lipgloss.NewStyle().
				Foreground(colorFg).
				Render(opt.Label)

			line := "  " + label
			if opt.Value != "" {
				path := lipgloss.NewStyle().
					Foreground(colorDim).
					Render("  " + opt.Value)
				line += path
			}
			b.WriteString(line + "\n")
		}
	}

	help := "\n" + FormatHelpLine("↑↓", "选择", "Enter", "确认", "Esc", "取消")
	b.WriteString(help)

	return DocStyle.Render(PanelStyle.Render(b.String()))
}

func (m rootSelectModel) viewInput() string {
	var b strings.Builder

	title := PanelTitleStyle.Render("输入根目录路径")
	b.WriteString(title + "\n\n")
	b.WriteString(m.input.View() + "\n")

	help := "\n" + FormatHelpLine("Enter", "确认", "Esc", "返回选择")
	b.WriteString(help)

	return DocStyle.Render(PanelStyle.Render(b.String()))
}

// ── 对外接口（签名不变）─────────────────────────────────────

func SelectRoot(label string, options []RootOption, defaultInput string) (string, bool, error) {
	if len(options) == 0 {
		options = []RootOption{
			{Label: "手动输入路径...", IsManual: true},
		}
	}

	m := newRootSelectModel(options, defaultInput)

	p := tea.NewProgram(m)
	finalModel, err := p.Run()
	if err != nil {
		return "", false, fmt.Errorf("根目录选择失败：%w", err)
	}

	result := finalModel.(rootSelectModel)
	if result.canceled {
		return "", true, nil
	}

	fmt.Println(SelectedStyle.Render("  ✓ 已选择: " + result.selected))
	fmt.Println()

	return result.selected, false, nil
}
