local function on_each_collision(col)
	local bounce_speed = 20.0
	local a_id = col.a
	local b_id = col.b

	local normal_x = col.normal[1]
	local normal_y = col.normal[2]
	local length = math.sqrt(normal_x ^ 2 + normal_y ^ 2)

	if length == 0 then
		normal_x = 1
		normal_y = 0
	else
		normal_x = normal_x / length
		normal_y = normal_y / length
	end


	if a_id == WORLD.player_id() or b_id == WORLD.player_id() then
		if a_id == WORLD.player_id() then
			--[[
			if CONFIG.entities[b_id].on_player_collision == "bounce" then
				engine.set_velocity_2d(
					a_id,
					-normal_x * bounce_speed,
					-normal_y * bounce_speed
				)
				ENGINE_HANDLES.set_state(WORLD.player_id(), GLOBALS.ACTIONS.Idle)
				CONTROLLER.start_input_reenable_timer(0.3)
			end
			if CONFIG.entities[b_id].on_collision == "bounce" then
				engine.set_velocity_2d(
					b_id,
					normal_x * bounce_speed,
					normal_y * bounce_speed
				)
				ENGINE_HANDLES.set_state(b_id, GLOBALS.ACTIONS.Idle)
			end
			]]
		end

		if
				(CONFIG.entities[a_id] and CONFIG.entities[a_id].layers[GLOBALS.MASKS_AND_LAYERS.Enemy])
				or (CONFIG.entities[b_id] and CONFIG.entities[b_id].layers[GLOBALS.MASKS_AND_LAYERS.Enemy])
		then
			if
					not (
						ENGINE_HANDLES.is_untargetable(a_id) or ENGINE_HANDLES.is_untargetable(b_id)
					)
			then
				ENGINE_HANDLES.mark_untargetable(WORLD.player_id(), 1)
				-- local dead = engine.damage(WORLD.player_id(), 2)
				if dead == true then
					ENGINE_HANDLES.set_state(WORLD.player_id(), GLOBALS.ACTIONS.Dying)
					WORLD.set_game_over()
					CONTROLLER.start_input_reenable_timer(100)
				end
			end
		end
	end
end

return { on_each_collision = on_each_collision }
