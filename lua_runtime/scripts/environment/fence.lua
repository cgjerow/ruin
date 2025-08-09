local aesprite_parser = require("aseprite_parser")

local function new_fence(x, y, w, h)
	return PhysicsBodyBuilder()
			:add_layer(GLOBALS.MASKS_AND_LAYERS.Env)
			:add_animation(GLOBALS.ACTIONS.Idle, aesprite_parser.load_aseprite_animation("fence", "arena/", "fence.json"))
			:size(w, h)
			:position(x, y)
			:body_type(GLOBALS.PHYSICS_BODIES.Static)
			:build()
end

return new_fence
