-- Hyprland config managed by SplinterDots.
-- Use SplinterDots for normal changes: dotctl center

require("colors")
require("dotfiles-generated")
require("keybindings")
require("user")
require("conf.splinter-tools-workspace")

hl.monitor({ output = "", mode = "preferred", position = "auto", scale = 1 })

hl.on("hyprland.start", function()
    hl.exec_cmd("~/.local/bin/splinter-autostart")
end)

hl.env("XCURSOR_SIZE", "24")
hl.env("XDG_CURRENT_DESKTOP", "Hyprland")

hl.window_rule({ name = "float-nautilus", match = { class = "^(org.gnome.Nautilus)$" }, float = true })
hl.window_rule({ name = "float-pavucontrol", match = { class = "^(pavucontrol)$" }, float = true })

hl.config({
    misc = {
        disable_hyprland_logo = true,
        disable_splash_rendering = true,
        force_default_wallpaper = 0,
    },
})
