local load_aseprite_animation = require("aseprite_parser")

local function new_fence(x, y)
	return ElementBuilder()
		:add_layer(GLOBALS.MASKS_AND_LAYERS.Env)
		:add_animation(GLOBALS.ACTIONS.Idle, load_aseprite_animation("fence", "arena/", "fence.json"))
		:size(2, 2)
		:position(x, y)
		:collider_size_modifier(3, 3)
		:build()
end

return new_fence
