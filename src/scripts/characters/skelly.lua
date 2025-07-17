local load_aseprite_animation = require("aseprite_parser")
require("game_asset_builders")
require("globals")

local function new_skelly(x, y)
	return PhysicsBodyBuilder()
		:add_layer(GLOBALS.MASKS_AND_LAYERS.Enemy)
		:add_mask(GLOBALS.MASKS_AND_LAYERS.Env)
		:add_mask(GLOBALS.MASKS_AND_LAYERS.Player)
		:add_animation(GLOBALS.ACTIONS.Idle, load_aseprite_animation("skelly_idle", "skelly/", "skelly_idle.json"))
		:size(2, 2)
		:position(x, y)
		:collider_size_modifier(0.3, 0.3)
		:build()
end

local function move_skellies()
	print("es")
	local player_p = engine.get_position_2d(STATE.player_id)
	print("no")
	for key, value in pairs(STATE.entities) do
		if value.is_skelly then
			local skelly_p = engine.get_position_2d(key)
			local ex, ey = skelly_p[1], skelly_p[2]
			local px, py = player_p[1], player_p[2]
			-- Direction vector from enemy to player
			local dx = px - ex
			local dy = py - ey

			-- Length (magnitude) of the direction vector
			local dist = math.sqrt(dx * dx + dy * dy)

			if dist > 0.001 then
				-- Normalize direction
				local nx = dx / dist
				local ny = dy / dist

				-- Movement speed or force
				local fx = nx * STATE.run_force
				local fy = ny * STATE.run_force
				engine.apply_force_2d(key, fx, fy)
			end
		end
	end
end

return { new = new_skelly, move = move_skellies }
