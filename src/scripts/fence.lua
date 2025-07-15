local load_aseprite_animation = require("aseprite_parser")

local IDLE, err = load_aseprite_animation("fence", "arena/", "fence.json")

local function new_fence(x, y)
	local s = {
		is_pc = false,
		x = x,
		y = y,
		width = 4, -- 1 = 32px
		height = 4,
		state = "Idle",
		animations = {
			Idle = IDLE,
		},
	}
	return s
end

return new_fence
