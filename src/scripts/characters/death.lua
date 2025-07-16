local load_aseprite_animation = require("aseprite_parser")
require("game_asset_builders")
require("globals")

local idle = load_aseprite_animation("death_idle", "death/", "death_idle.json")
local running = load_aseprite_animation("death_running", "death/", "death_running.json")
local dying = load_aseprite_animation("death_dying", "death/", "death_dying.json")

local function summon_death(x, y)
	return ElementBuilder()
		:add_layer(GLOBALS.MASKS_AND_LAYERS.Player)
		:add_mask(GLOBALS.MASKS_AND_LAYERS.Env)
		:add_mask(GLOBALS.MASKS_AND_LAYERS.Enemy)
		:add_animation(GLOBALS.ACTIONS.Idle, idle)
		:add_animation(GLOBALS.ACTIONS.Running, running)
		:add_animation(GLOBALS.ACTIONS.Dying, dying)
		:size(2, 2)
		:position(x, y)
		:collider_size_modifier(0.8, 0.8)
		:build()
end

return summon_death
