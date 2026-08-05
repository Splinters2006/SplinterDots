-- Splinter pinned tool overlays
-- SUPER CTRL + T toggles Splinter tools
-- SUPER SHIFT + T pins/unpins the current focused window

hl.bind("SUPER + CTRL + T", hl.dsp.exec_cmd("~/.local/bin/splinter-toggle-tools"))
hl.bind("SUPER + SHIFT + T", hl.dsp.window.pin())
