local load_aseprite_animation = require("aseprite_parser")
local pretty_print = require("pretty_print")

local idle = load_aseprite_animation("death_idle", "death/", "death_idle.json")
local running = load_aseprite_animation("death_running", "death/", "death_running.json")

pretty_print(running)

DEATH = {
	is_pc = true,
	x = 0,
	y = 0,
	width = 2, -- 1 = 32px
	height = 2,
	state = "Idle",
	animations = {
		Idle = idle,
		Running = running,
	},
}
