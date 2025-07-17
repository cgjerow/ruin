local load_aseprite_animation = require("aseprite_parser")

local function new_fence(x, y)
	return PhysicsBodyBuilder()
		:add_layer(GLOBALS.MASKS_AND_LAYERS.Env)
		:add_animation(GLOBALS.ACTIONS.Idle, load_aseprite_animation("fence", "arena/", "fence.json"))
		:size(2, 2)
		:position(x, y)
		:body_type(GLOBALS.PHYSICS_BODIES.Static)
		:build()
end

return new_fence
