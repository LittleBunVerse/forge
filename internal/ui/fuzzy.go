// Package ui 提供终端交互：目录模糊选择与启动模式选择。
package ui

import (
	"fmt"
	"io"
	"strings"

	"github.com/LittleBunVerse/forge/internal/scan"

	"github.com/charmbracelet/bubbles/list"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

// ── list.Item 适配 ──────────────────────────────────────────

type dirItem struct {
	dir scan.Dir
}

func (d dirItem) Title() string       { return d.dir.Name }
func (d dirItem) Description() string { return d.dir.Path }
func (d dirItem) FilterValue() string { return d.dir.Name }

// ── Bubble Tea Model ────────────────────────────────────────

type dirSelectModel struct {
	list     list.Model
	selected scan.Dir
	canceled bool
	done     bool
}

func (m dirSelectModel) Init() tea.Cmd {
	return nil
}

func (m dirSelectModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.KeyMsg:
		// 过滤模式下不拦截 esc / q，交给 list 处理
		if m.list.FilterState() == list.Filtering {
			break
		}
		switch msg.String() {
		case "enter":
			if item, ok := m.list.SelectedItem().(dirItem); ok {
				m.selected = item.dir
				m.done = true
				return m, tea.Quit
			}
		case "esc", "ctrl+c":
			m.canceled = true
			m.done = true
			return m, tea.Quit
		}
	case tea.WindowSizeMsg:
		h, v := DocStyle.GetFrameSize()
		m.list.SetSize(msg.Width-h, msg.Height-v)
	}

	var cmd tea.Cmd
	m.list, cmd = m.list.Update(msg)
	return m, cmd
}

func (m dirSelectModel) View() string {
	if m.done {
		return ""
	}
	return DocStyle.Render(m.list.View())
}

// ── 自定义 Delegate ─────────────────────────────────────────

type dirDelegate struct {
	list.DefaultDelegate
}

func newDirDelegate() dirDelegate {
	d := list.NewDefaultDelegate()

	// 样式自定义
	d.Styles.SelectedTitle = lipgloss.NewStyle().
		Foreground(colorSecondary).
		Bold(true).
		PaddingLeft(1)

	d.Styles.SelectedDesc = lipgloss.NewStyle().
		Foreground(colorDim).
		PaddingLeft(1)

	d.Styles.NormalTitle = lipgloss.NewStyle().
		Foreground(colorFg).
		PaddingLeft(2)

	d.Styles.NormalDesc = lipgloss.NewStyle().
		Foreground(colorDim).
		PaddingLeft(2)

	d.SetSpacing(0)

	return dirDelegate{DefaultDelegate: d}
}

func (d dirDelegate) Render(w io.Writer, m list.Model, index int, item list.Item) {
	i, ok := item.(dirItem)
	if !ok {
		return
	}

	isSelected := index == m.Index()

	var title, desc string
	if isSelected {
		title = d.Styles.SelectedTitle.Render("▌ " + i.Title())
		desc = d.Styles.SelectedDesc.Render("  " + i.Description())
	} else {
		title = d.Styles.NormalTitle.Render("  " + i.Title())
		desc = d.Styles.NormalDesc.Render("  " + i.Description())
	}

	fmt.Fprintf(w, "%s\n%s", title, desc)
}

func (d dirDelegate) Height() int  { return 2 }
func (d dirDelegate) Spacing() int { return 0 }

// ── 对外接口（签名不变）─────────────────────────────────────

func SelectDir(dirs []scan.Dir) (scan.Dir, bool, error) {
	items := make([]list.Item, len(dirs))
	for i, d := range dirs {
		items[i] = dirItem{dir: d}
	}

	delegate := newDirDelegate()

	l := list.New(items, delegate, 0, 0)
	l.Title = "选择项目目录"
	l.SetFilteringEnabled(true)
	l.SetShowStatusBar(true)
	l.SetShowHelp(true)

	// 标题样式
	l.Styles.Title = lipgloss.NewStyle().
		Bold(true).
		Foreground(colorPrimary).
		Padding(0, 1)

	// 过滤提示样式
	l.FilterInput.PromptStyle = lipgloss.NewStyle().Foreground(colorSecondary)
	l.FilterInput.TextStyle = lipgloss.NewStyle().Foreground(colorFg)

	// 状态栏
	l.Styles.StatusBar = lipgloss.NewStyle().Foreground(colorDim)

	// 帮助信息样式
	helpStyle := list.DefaultStyles().HelpStyle
	helpStyle = helpStyle.PaddingLeft(0).PaddingBottom(0)
	l.Styles.HelpStyle = helpStyle

	// 根据列表大小动态设置分页
	height := min(len(dirs)+8, 20)
	l.SetSize(60, height)

	m := dirSelectModel{list: l}
	p := tea.NewProgram(m, tea.WithAltScreen())

	finalModel, err := p.Run()
	if err != nil {
		return scan.Dir{}, false, fmt.Errorf("目录选择失败：%w", err)
	}

	result := finalModel.(dirSelectModel)
	if result.canceled {
		return scan.Dir{}, true, nil
	}

	// 输出选择结果
	fmt.Println(SelectedStyle.Render("✓ 已选择: " + result.selected.Name))
	fmt.Println(SubtitleStyle.Render("  " + result.selected.Path))
	fmt.Println()

	return result.selected, false, nil
}

func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}

// ── 简单搜索 helper（给 root.go 等复用）────────────────────

func fuzzyMatch(input, target string) bool {
	input = strings.ToLower(input)
	target = strings.ToLower(target)
	return strings.Contains(target, input)
}
