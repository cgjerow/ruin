local load_aseprite_animation = require("aseprite_parser")
require("game_asset_builders")
require("globals")

local function new_skelly(x, y)
	return ElementBuilder()
		:add_layer(GLOBALS.MASKS_AND_LAYERS.Enemy)
		:add_mask(GLOBALS.MASKS_AND_LAYERS.Env)
		:add_mask(GLOBALS.MASKS_AND_LAYERS.Player)
		:add_animation(GLOBALS.ACTIONS.Idle, load_aseprite_animation("skelly_idle", "skelly/", "skelly_idle.json"))
		:size(2, 2)
		:position(x, y)
		:build()
end

return new_skelly
