# 0003 macOS Full-Screen Overlay Activation Policy

## 状态

Accepted

## 背景

macOS 全屏应用运行在独立 Space 中。普通 Tauri `WebviewWindow` 即使设置 `alwaysOnTop` 和 `visibleOnAllWorkspaces`，仍可能无法显示在全屏应用之上。

Agent Island 的悬浮岛更接近 HUD / 菜单栏辅助应用，而不是常规文档型应用。用户可以接受不在 Dock 和 App Switcher 中保留主应用图标。

## 决策

macOS 下在 Tauri `setup` 阶段设置应用激活策略为 `tauri::ActivationPolicy::Accessory`，再配置悬浮岛窗口：

- 主窗口继续使用 Tauri `WebviewWindow`。
- 窗口继续设置 `alwaysOnTop`、`visibleOnAllWorkspaces`、`FullScreenAuxiliary`、`CanJoinAllSpaces` 和高窗口层级。
- 不把现有 Tauri `NSWindow` 直接改成 `NonactivatingPanel` style；该方式不适用于现有窗口，并且曾导致启动崩溃。
- 如果 Accessory 激活策略仍无法满足全屏展示，再进入原生 AppKit `NSPanel` / status item HUD 方案。

## 后果

好处：

- 更符合跨全屏 Space 的悬浮辅助应用模型。
- 改动范围小，仍保留现有 Tauri UI 和状态管理。
- 避免在现有 Tauri `NSWindow` 上强行叠加不支持的 panel style。

代价：

- macOS Dock 和 App Switcher 中不再显示 Agent Island 主应用图标。
- 这不是最终保证；不同 macOS 版本和全屏应用仍可能需要原生 `NSPanel`。
- 该行为只能在原生 Tauri app 中验证，浏览器预览无法验证。
