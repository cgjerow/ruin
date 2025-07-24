local load_aseprite_animation = require("aseprite_parser")
require("game_asset_builders")
require("globals")

local is_transparent = true
local idle = load_aseprite_animation("death_idle", "death/", "death_idle.json", is_transparent)
local running = load_aseprite_animation("death_running", "death/", "death_running.json", is_transparent)
local dying = load_aseprite_animation("death_dying", "death/", "death_dying.json", is_transparent)
local dashing = load_aseprite_animation("death_blinking", "death/", "death_blinking.json", is_transparent)

local function summon_death(x, y)
	return PhysicsBodyBuilder()
			:position(x, y)
			:size(2, 2)
			:collider_size_modifier(0.6, 0.8)
			:add_layer(GLOBALS.MASKS_AND_LAYERS.Player)
			:add_mask(GLOBALS.MASKS_AND_LAYERS.Env)
			:add_mask(GLOBALS.MASKS_AND_LAYERS.Enemy)
			:add_animation(GLOBALS.ACTIONS.Idle, idle)
			:add_animation(GLOBALS.ACTIONS.Dashing, dashing)
			:add_animation(GLOBALS.ACTIONS.Running, running)
			:add_animation(GLOBALS.ACTIONS.Dying, dying)
			:build()
end

return summon_death
