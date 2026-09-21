---@diagnostic disable-next-line: lowercase-global
function main()
	return {
		debug_enabled = true,
		fps = "auto", -- Default auto, set as auto or a number for specific frame rate target
		window_width = 1280,
		window_height = 720,
		virtual_resolution_width = 320,
		virtual_resolution_height = 160,
		-- Physics range in world units (radius of collision detection around player)
		-- Lower = faster collision but entities farther away won't collide
		-- Higher = slower collision but better simulation coverage
		physics_range = 30.0,
		-- Physics simulation frequency (Hz). 60 is recommended for performance.
		-- 300 was the old default - way too high for 1000 entities
		physics_fps = 60,
		camera_config = {
			zoom = 15.0,
			initial_pos_x = 0.0,
			initial_pos_y = 0.0,
			look_ahead_smooth_factor = 10.0,
			look_ahead_distance = 10.0,
			look_ahead_lerp_speed = 2.0,
		},
	}
end
