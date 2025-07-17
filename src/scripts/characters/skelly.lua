local load_aseprite_animation = require("aseprite_parser")
require("game_asset_builders")
require("globals")

local function new_skelly(x, y)
	return PhysicsBodyBuilder()
			:position(x, y)
			:size(2, 2)
			:add_layer(GLOBALS.MASKS_AND_LAYERS.Enemy)
			:add_mask(GLOBALS.MASKS_AND_LAYERS.Env)
			:add_mask(GLOBALS.MASKS_AND_LAYERS.Enemy)
			:add_mask(GLOBALS.MASKS_AND_LAYERS.Player)
			:collider_size_modifier(0.3, 0.3)
			:add_animation(GLOBALS.ACTIONS.Idle, load_aseprite_animation("skelly_idle", "skelly/", "skelly_idle.json"))
			:build()
end

local function move_skellies()
	local player_p = engine.get_position_2d(WORLD.player_id())
	for key, value in pairs(CONFIG.entities) do
		if value.is_skelly then
			local random_action = math.random(1, 10)
			local skelly_p = engine.get_position_2d(key)
			local ex, ey = skelly_p[1], skelly_p[2]
			local px, py = player_p[1], player_p[2]
			-- Direction vector from enemy to player
			local dx = px - ex
			local dy = py - ey

			ENGINE_HANDLES.flip_x(key, dx)

			-- Length (magnitude) of the direction vector
			local dist = math.sqrt(dx * dx + dy * dy)
			if dist < 0.001 then
				return
			end

			-- Normalize direction
			local nx = dx / dist
			local ny = dy / dist
			if random_action < 9 and dist < 4 and not ENGINE_HANDLES.is_untargetable(WORLD.player_id()) then
				-- Movement speed or force
				local fx = nx * 10
				local fy = ny * 10
				engine.apply_impulse_2d(key, fx, fy)
			elseif random_action < 9 then
				-- Movement speed or force
				local fx = nx * 50
				local fy = ny * 50
				engine.apply_force_2d(key, fx, fy)
			end
		end
	end
end

return { new = new_skelly, move = move_skellies }
