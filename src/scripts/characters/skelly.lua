local load_aseprite_animation = require("aseprite_parser")
require("game_asset_builders")
require("globals")

local is_transparent = true
local function new_skelly(x, y)
	return PhysicsBodyBuilder()
			:position(x, y)
			:size(2, 2)
			:add_layer(GLOBALS.MASKS_AND_LAYERS.Enemy)
			:add_mask(GLOBALS.MASKS_AND_LAYERS.Env)
			:add_mask(GLOBALS.MASKS_AND_LAYERS.Enemy)
			:add_mask(GLOBALS.MASKS_AND_LAYERS.Player)
			:collider_size_modifier(0.3, 0.3)
			:add_animation(GLOBALS.ACTIONS.Idle,
				load_aseprite_animation("skelly_idle", "skelly/", "skelly_idle.json", is_transparent))
			:build()
end

local function move_skellies(dt)
	local player_p = engine.get_position_2d(WORLD.player_id())
	for key, value in pairs(CONFIG.entities) do
		if not value.is_skelly then goto continue end

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

		if WORLD.activity_state[key] and WORLD.activity_state[key].activity == "lunge" then
			WORLD.activity_state[key].time = WORLD.activity_state[key].time - dt
			engine.set_velocity_2d(key, WORLD.activity_state[key].direction_x, WORLD.activity_state[key].direction_y)
			if WORLD.activity_state[key].time <= 0 then
				WORLD.activity_state[key].activity = "pursuing"
				engine.set_velocity_2d(key, 0, 0)
				print("clear", WORLD.activity_state[key].activity)
			end
			goto continue
		end

		if WORLD.activity_state[key] and WORLD.activity_state[key].activity == "lunge-ramping" then
			WORLD.activity_state[key].time = WORLD.activity_state[key].time - dt
			if WORLD.activity_state[key].time <= 0 then
				local fx = nx * 50
				local fy = ny * 50
				WORLD.activity_state[key].activity = "lunge"
				WORLD.activity_state[key].time = .3
				WORLD.activity_state[key].direction_x = fx
				WORLD.activity_state[key].direction_y = fy

				engine.set_velocity_2d(key, fx, fy)

				print("clear", WORLD.activity_state[key].activity)
			end
			goto continue
		end


		if (not WORLD.activity_state[key] or WORLD.activity_state[key].activity == "pursuing") then
			local should_lunge = dist < 8
			if should_lunge then
				WORLD.activity_state[key] = { activity = "lunge-ramping", time = 2 }
				engine.set_velocity_2d(key, 0, 0)
			else
				local fx = nx * 10
				local fy = ny * 10
				engine.set_velocity_2d(key, fx, fy)
			end
			goto continue
		end


		::continue::
	end
end

return { new = new_skelly, move = move_skellies }
