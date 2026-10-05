# Project Overview

Tuimux is a Rust terminal UI for browsing and managing tmux sessions, windows, and panes from a full-screen dashboard.

The goal is to make common tmux workflows faster:
- inspect all sessions at a glance
- understand window and pane structure quickly
- preview sessions and windows
- create, attach to, rename, and close sessions all from the TUI. 

## Requirements

- Create full screen TUI dashboard of all tmux sessions and windows.
- Should be able to create new tmux sessions/windows easily with keybindings
- Should be able to attach to tmux sessions/windows easily with keybindings
- Should show a list of existing tmux sessions that, when hovered, show a preview of the tab
- Easy to see nested windows underneath sessions.
- Should be able to delete windows/sessions easily with keybindings.

## Architecture

- Use Ratatui for TUI handling
- I have included tmux bindings from another Rust project that should get us most of the way (./src/tmux/*)

## UX Notes

The initial screen should likely have three regions:
- Left: session tree
  - sessions
  - nested windows

- Right: preview/details
  - selected session name
  - working directory
  - windows and pane commands
  - focused item metadata

- Bottom: keybinding help
  - create session
  - create window
  - attach
  - rename
  - close
  - quit

