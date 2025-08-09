local aesprite_parser = require("aseprite_parser")

local function main_menu(w, h)
	return CanvasSceneBuilder()
			:add_animation(GLOBALS.ACTIONS.Idle,
				aesprite_parser.load_aseprite_animation("canvas/", "canvas/", "main_menu.json"))
			:size(w, h)
			:position(0, 0)
			:build()
end

return main_menu
