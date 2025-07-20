local load_aseprite_animation = require("aseprite_parser")

local function new_brick_tile(x, y)
	return PhysicsBodyBuilder()
			:add_animation(GLOBALS.ACTIONS.Idle, load_aseprite_animation("bricks_1", "/", "bricks.json"))
			:size(2, 2)
			:position(x, y)
			:build()
end

return new_brick_tile
