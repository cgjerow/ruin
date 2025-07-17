local load_aseprite_animation = require("aseprite_parser")
require("game_asset_builders")
require("globals")

local function new_skelly(x, y)
	return PhysicsBodyBuilder()
		:add_layer(GLOBALS.MASKS_AND_LAYERS.Enemy)
		:add_mask(GLOBALS.MASKS_AND_LAYERS.Env)
		:add_mask(GLOBALS.MASKS_AND_LAYERS.Player)
		:add_animation(GLOBALS.ACTIONS.Idle, load_aseprite_animation("skelly_idle", "skelly/", "skelly_idle.json"))
		:size(2, 2)
		:position(x, y)
		:collider_size_modifier(0.3, 0.3)
		:build()
end

return new_skelly
