local load_aseprite_animation = require("aseprite_parser")

local IDLE, err = load_aseprite_animation("skelly_idle", "skelly/", "skelly_idle.json")

local function new_skelly(x, y)
	local s = {
		is_pc = false,
		x = x,
		y = y,
		width = 2, -- 1 = 32px
		height = 2,
		state = "Idle",
		animations = {
			Idle = IDLE,
		},
	}
	return s
end

return new_skelly
