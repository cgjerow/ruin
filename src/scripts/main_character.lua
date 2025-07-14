local load_aseprite_animation = require("aseprite_parser")
local pretty_print = require("pretty_print")

local idle = load_aseprite_animation("death_idle", "death/", "death_idle.json")
print("idle: ")
pretty_print(idle)

DEATH = {
	is_pc = true,
	x = 0,
	y = 0,
	width = 4, -- 1 = 32px
	height = 4,
	state = "Idle",
	animations = {
		Idle = idle,
	},
}
