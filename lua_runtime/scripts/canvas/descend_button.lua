local aseprite_parser = require("aseprite_parser")

local button = aseprite_parser.load_stateful_ui("canvas/", "canvas_theme")

local function descend_button()
	return CanvasSceneBuilder()
			:add_animation(GLOBALS.UI_STATE.Default, button.animations.default)
			:add_animation(GLOBALS.UI_STATE.Hovered, button.animations.hovered)
			:add_animation(GLOBALS.UI_STATE.Pressed, button.animations.pressed)
			:size(button.texture_w, button.texture_h)
			:position(0, 0)
			:build()
end

return descend_button
