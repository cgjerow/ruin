local load_aseprite_animation = require("aseprite_parser")

local function new_fence(x, y, w, h)
	return PhysicsBodyBuilder()
			:add_layer(GLOBALS.MASKS_AND_LAYERS.Env)
			:add_animation(GLOBALS.ACTIONS.Idle, load_aseprite_animation("fence", "arena/", "fence.json"))
			:size(w, h)
			:position(x, y)
			:body_type(GLOBALS.PHYSICS_BODIES.Static)
			:build()
end

return new_fence
