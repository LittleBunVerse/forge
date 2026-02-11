package ui

import (
	"fmt"
	"os"
	"strings"

	"github.com/LittleBunVerse/forge/internal/config"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

// WorkspaceChoice 表示用户选择的工作区来源。
type WorkspaceChoice int

const (
	WorkspaceCurrentDir WorkspaceChoice = iota // 使用当前目录
	WorkspaceProject                           // 使用项目书签
	WorkspaceBrowseRoot                        // 从已保存的根目录浏览子文件夹
	WorkspaceNewRoot                           // 选择新的根目录
)

// WorkspaceResult 封装选择结果。
type WorkspaceResult struct {
	Choice      WorkspaceChoice
	ProjectPath string // CurrentDir / Project 时填充目标路径
}

// ── 内部选项 ────────────────────────────────────────────────

type wsOption struct {
	label string
	desc  string
	value WorkspaceChoice
	path  string // 直接可用的路径（CurrentDir / Project）
}

// ── 序号徽章 ────────────────────────────────────────────────

var numberBadges = []string{"❶", "❷", "❸", "❹", "❺", "❻", "❼", "❽", "❾", "❿"}

func getBadge(index int) string {
	if index < len(numberBadges) {
		return numberBadges[index]
	}
	return fmt.Sprintf("%d.", index+1)
}

// ── Bubble Tea Model ────────────────────────────────────────

type workspaceModel struct {
	options  []wsOption
	cursor   int
	selected wsOption
	canceled bool
	done     bool
}

func (m workspaceModel) Init() tea.Cmd { return nil }

func (m workspaceModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
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
			m.selected = m.options[m.cursor]
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

func (m workspaceModel) View() string {
	if m.done {
		return ""
	}

	var b strings.Builder

	// 面板标题
	title := PanelTitleStyle.Render("选择工作区")
	b.WriteString(title + "\n\n")

	for i, opt := range m.options {
		if i == m.cursor {
			cursor := CursorBar.String()
			arrow := lipgloss.NewStyle().Foreground(colorCursorBar).Bold(true).Render(" ›")

			label := lipgloss.NewStyle().
				Bold(true).
				Foreground(colorSecondary).
				Render(opt.label)

			line := cursor + " " + label + arrow
			if opt.desc != "" {
				desc := SubtitleStyle.Render("  " + opt.desc)
				line += desc
			}
			b.WriteString(line + "\n")
		} else {
			label := lipgloss.NewStyle().
				Foreground(colorFg).
				Render(opt.label)

			line := "  " + label
			if opt.desc != "" {
				desc := lipgloss.NewStyle().
					Foreground(colorDim).
					Render("  " + opt.desc)
				line += desc
			}
			b.WriteString(line + "\n")
		}
	}

	// 快捷键帮助
	help := "\n" + FormatHelpLine("↑↓", "选择", "Enter", "确认", "Esc", "取消")
	b.WriteString(help)

	return DocStyle.Render(PanelStyle.Render(b.String()))
}

// ── 对外接口 ────────────────────────────────────────────────

// SelectWorkspace 显示统一的工作区选择 TUI。
// savedRoot: 已保存的根目录（空则不显示"从根目录浏览"选项）。
// projects: 用户配置的项目书签列表。
func SelectWorkspace(savedRoot string, projects []config.ProjectConfig) (WorkspaceResult, bool, error) {
	cwd, _ := os.Getwd()

	var options []wsOption

	// 选项 1：使用当前目录
	options = append(options, wsOption{
		label: "当前目录",
		desc:  cwd,
		value: WorkspaceCurrentDir,
		path:  cwd,
	})

	// 选项 2-N：项目书签
	for _, p := range projects {
		options = append(options, wsOption{
			label: p.Name,
			desc:  p.Path,
			value: WorkspaceProject,
			path:  p.Path,
		})
	}

	// 选项 N+1：从已保存的根目录浏览子文件夹
	if strings.TrimSpace(savedRoot) != "" {
		options = append(options, wsOption{
			label: "从根目录选择子文件夹…",
			desc:  savedRoot,
			value: WorkspaceBrowseRoot,
		})
	}

	// 选项 N+2：选择新的根目录
	options = append(options, wsOption{
		label: "选择新的根目录…",
		value: WorkspaceNewRoot,
	})

	m := workspaceModel{options: options}
	p := tea.NewProgram(m)

	finalModel, err := p.Run()
	if err != nil {
		return WorkspaceResult{}, false, fmt.Errorf("工作区选择失败：%w", err)
	}

	result := finalModel.(workspaceModel)
	if result.canceled {
		return WorkspaceResult{}, true, nil
	}

	sel := result.selected
	return WorkspaceResult{
		Choice:      sel.value,
		ProjectPath: sel.path,
	}, false, nil
}
