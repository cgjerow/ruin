local total = {
	frame = 0,
	decel = 0,
}

local function apply_drag_to_rigids(dt)
	for id, elem in pairs(CONFIG.entities) do
		if elem.type == GLOBALS.PHYSICS_BODIES.Rigid then
			local vel = engine.get_velocity_2d(id) -- { vx, vy }

			local drag = CONFIG.friction * dt
			if drag < CONFIG.min_friction * dt then
				drag = CONFIG.min_friction * dt
			end

			local speed = math.sqrt(vel[1] ^ 2 + vel[2] ^ 2)
			if total.frame > 1 and id == WORLD.player_id() then
				print("decel ", total.decel)
				total.frame = 0
				total.decel = 0
			elseif id == WORLD.player_id() then
				total.frame = total.frame + dt
				total.decel = total.decel + drag
			end

			if id == WORLD.player_id() and speed > 40 then
				print("v before", vel[1], vel[2])
			end


			-- Damping per component, clamped to avoid crossing zero
			for i = 1, 2 do
				local v = vel[i]
				local damped = v * math.exp(-drag * dt)
				-- If it would flip sign, clamp to 0 instead
				if v > 0 and damped < 0 then
					vel[i] = 0
				elseif v < 0 and damped > 0 then
					vel[i] = 0
				else
					vel[i] = damped
				end
			end

			-- 2) Enforce a max speed if not in a special state (e.g. dashing)
			speed = math.sqrt(vel[1] ^ 2 + vel[2] ^ 2)
			if id == WORLD.player_id() and speed > 40 then
				print("v vafter", vel[1], vel[2])
			end
			local max
			if id == WORLD.player_id() then max = CONFIG.max_speed else max = CONFIG.skelly_speed end
			if speed > max then
				local scale = max / speed
				vel[1] = vel[1] * scale
				vel[2] = vel[2] * scale
			end

			speed = math.sqrt(vel[1] ^ 2 + vel[2] ^ 2)
			engine.set_velocity_2d(id, vel[1], vel[2])

			local idle = 1
			if speed < idle then
				ENGINE_HANDLES.set_state(id, GLOBALS.ACTIONS.Idle)
			end
		end
	end
end

return { apply_drag_to_rigids = apply_drag_to_rigids }
