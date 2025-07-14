local load_aseprite_animation = require("aseprite_parser")
local pretty_print = require("pretty_print")

local idle = load_aseprite_animation("death_idle", "death/", "death_idle.json")

DEATH = {
	x = 0,
	y = 0,
	width = 1, -- 1 = 32px
	height = 1,
	state = "Idle",
	animations = {
		Idle = idle,
	},
}
