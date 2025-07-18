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

local function move_skellies(dt)
	local player_p = engine.get_position_2d(WORLD.player_id())
	for key, value in pairs(CONFIG.entities) do
		if value.is_skelly then
			if WORLD.activity_state[key] and WORLD.activity_state[key].activity == "lunge" then
				WORLD.activity_state[key].time = WORLD.activity_state[key].time - dt
				if WORLD.activity_state[key].time <= 0 then
					WORLD.activity_state[key].activity = "pursuing"
					print("clear", WORLD.activity_state[key].activity)
				end
			end

			local random_action = math.random(1, 12)
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

			local not_lunging = (not WORLD.activity_state[key]) or (WORLD.activity_state[key].activity ~= "lunge")
			local should_lunge = not_lunging and random_action < 9 and dist < 10 and
			not ENGINE_HANDLES.is_untargetable(WORLD.player_id())


			if should_lunge then
				local fx = nx * 100
				local fy = ny * 100
				WORLD.activity_state[key] = { activity = "lunge", time = 2 }
				engine.set_velocity_2d(key, fx, fy)
			elseif random_action < 9 then
				local fx = nx * 10
				local fy = ny * 10
				engine.set_velocity_2d(key, fx, fy)
			end
		end
	end
end

return { new = new_skelly, move = move_skellies }
