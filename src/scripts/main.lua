---@diagnostic disable: unused-function, lowercase-global
---@diagnostic disable-next-line: unused-local
local pretty_print = require("pretty_print")

-- Game Elements
local summon_death = require("characters.death")
local skelly = require("characters.skelly")
pretty_print(skelly)
local new_fence = require("environment.fence")
local new_skelly = skelly.new
local move_skellies = skelly.move

math.randomseed(os.time())

STATE = {
	max_speed = 10,
	skelly_max_speed = 5,
	skelly_speed = 8,
	dead = false,
	friction = 10,
	min_friction = .1,
	input_enabled = true,
	input_disable_time = 0,
	run_force = 200.0,
	player_id = -1,
	entities = {},
	untargetable = {},
	controller = ControllerBuilder()
			:key("SPACE", "Dash")
			:key("W", "Up")
			:key("S", "Down")
			:key("A", "MoveLeft")
			:key("D", "MoveRight")
			:build(),
}

function normalize(x, y)
	local mag = math.sqrt(x ^ 2 + y ^ 2)
	if mag == 0 then
		return 0, 0
	end
	return x / mag, y / mag
end

function table.clone(tbl)
	local copy = {}
	for k, v in pairs(tbl) do
		if type(v) == "table" then
			copy[k] = table.clone(v)
		else
			copy[k] = v
		end
	end
	return copy
end

function start_input_reenable_timer(seconds)
	if not STATE.input_disable_time or STATE.input_disable_time < seconds then
		STATE.input_enabled = false
		STATE.input_disable_time = seconds
	end
end

function keyboard_event(key, is_pressed, mouse_position)
	key = string.upper(key)
	STATE.controller:update(key, is_pressed, mouse_position)
end

function set_state(id, state)
	if not id == STATE.player_id or not STATE.dead then
		engine.set_state(id, state)
	end
end

function load()
	local death = summon_death(0, 0)
	death.on_collision = "bounce"
	STATE.player = death
	STATE.player_id = engine.create_body(death)
	death.id = STATE.player_id
	STATE.entities[STATE.player_id] = death

	local build_walls = true
	if build_walls then
		for i = 0, 50 do
			if i % 2 == 0 then
				goto continue
			end
			local fence = new_fence(i - 25, -25)
			fence.on_player_collision = "block"
			fence.on_collision = ""
			fence.id = engine.create_body(fence)
			STATE.entities[fence.id] = fence

			fence = new_fence(i - 25, 25)
			fence.on_player_collision = "block"
			fence.on_collision = ""
			fence.id = engine.create_body(fence)
			STATE.entities[fence.id] = fence

			fence = new_fence(-25, i - 25)
			fence.on_player_collision = "block"
			fence.on_collision = ""
			fence.id = engine.create_body(fence)
			STATE.entities[fence.id] = fence

			fence = new_fence(25, i - 25)
			fence.on_player_collision = "block"
			fence.on_collision = ""
			fence.id = engine.create_body(fence)
			STATE.entities[fence.id] = fence
			::continue::
		end
	end

	local build_skellys = true
	if build_skellys then
		for _ = 1, 10 do
			local x = math.random(10, 20)
			local y = math.random(10, 20)
			local flip_x = math.random(0, 1)
			local flip_y = math.random(0, 1)
			if flip_x == 1 then
				y = y * -1
			end
			if flip_y == 1 then
				x = x * -1
			end
			local s = new_skelly(x, y)
			-- s.is_pc = true
			s.id = engine.create_body(s)
			s.on_player_collision = "bounce"
			s.on_collision = "bounce"
			s.is_skelly = true
			STATE.entities[s.id] = s
			-- STATE.player = new_id
		end
	end

	return {
		assets = {},
	}
end

function on_each_collision(col)
	local bounce_speed = 50.0
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


	if a_id == STATE.player_id or b_id == STATE.player_id then
		if a_id == STATE.player_id then
			if STATE.entities[b_id].on_player_collision == "bounce" then
				engine.apply_impulse_2d(
					a_id,
					-normal_x * bounce_speed,
					-normal_y * bounce_speed
				)
				set_state(STATE.player_id, GLOBALS.ACTIONS.Idle)
				start_input_reenable_timer(0.3)
			end
			if STATE.entities[b_id].on_collision == "bounce" then
				engine.apply_impulse_2d(
					b_id,
					normal_x * bounce_speed,
					normal_y * bounce_speed
				)
				set_state(b_id, GLOBALS.ACTIONS.Idle)
			end
		end

		if
				(STATE.entities[a_id] and STATE.entities[a_id].layers[GLOBALS.MASKS_AND_LAYERS.Enemy])
				or (STATE.entities[b_id] and STATE.entities[b_id].layers[GLOBALS.MASKS_AND_LAYERS.Enemy])
		then
			if
					not (
						(STATE.untargetable[a_id] and STATE.untargetable[a_id] > 0)
						or (STATE.untargetable[b_id] and STATE.untargetable[b_id] > 0)
					)
			then
				STATE.untargetable[STATE.player_id] = 1
				local dead = engine.damage(STATE.player_id, 2)
				if dead == true then
					set_state(STATE.player_id, GLOBALS.ACTIONS.Dying)
					STATE.dead = true
					start_input_reenable_timer(100)
				end
			end
		end
	end
end

function on_collision(collisions)
	for _, col in ipairs(collisions) do
		on_each_collision(col)
	end
end

function update(dt)
	local dx, dy = 0, 0
	if STATE.untargetable[STATE.player_id] and STATE.untargetable[STATE.player_id] > 0 then
		STATE.untargetable[STATE.player_id] = STATE.untargetable[STATE.player_id] - dt
	end

	if not STATE.input_enabled then
		STATE.input_disable_time = STATE.input_disable_time - dt
		if STATE.input_disable_time <= 0 then
			STATE.input_enabled = true
		end
		return
	end

	if STATE.controller:is_pressed("Dash") then
		start_input_reenable_timer(0.3)
		local v = engine.get_velocity_2d(STATE.player_id)
		local x, y = normalize(v[1], v[2])

		if not (x == 0 and y == 0) then
			local impulse_strength = 100
			engine.apply_impulse_2d(STATE.player_id, x * impulse_strength, y * impulse_strength)
		end
	end

	if STATE.controller:is_pressed("Up") then
		dy = dy + 1
	end
	if STATE.controller:is_pressed("Down") then
		dy = dy - 1
	end
	if STATE.controller:is_pressed("MoveLeft") then
		dx = dx - 1
	end
	if STATE.controller:is_pressed("MoveRight") then
		dx = dx + 1
	end

	-- Normalize direction vector if needed
	local length = math.sqrt(dx * dx + dy * dy)
	if length > 0 then
		dx = dx / length
		dy = dy / length
		engine.apply_force_2d(STATE.player_id, dx * STATE.run_force, dy * STATE.run_force)
		set_state(STATE.player_id, GLOBALS.ACTIONS.Running)
		if math.abs(dx) > 0.01 then
			engine.flip(STATE.player_id, dx >= 0, false)
		end
	end

	-- move_skellies()
end


function getState()
	return STATE
end

function apply_drag_to_rigids(dt)
	for id, elem in pairs(STATE.entities) do
		if elem.type == GLOBALS.PHYSICS_BODIES.Rigid then
			local vel = engine.get_velocity_2d(id) -- { vx, vy }

			local decel = STATE.friction * dt
			if decel < STATE.min_friction then
				decel = STATE.min_friction
			end

			-- Damping per component, clamped to avoid crossing zero
			for i = 1, 2 do
				local v = vel[i]
				local damped = v * (1 - decel)
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
			local speed = math.sqrt(vel[1] ^ 2 + vel[2] ^ 2)
			local max
			if id == STATE.player_id then max = STATE.max_speed else max = STATE.skelly_speed end
			if speed > max then
				local scale = max / speed
				vel[1] = vel[1] * scale
				vel[2] = vel[2] * scale
			end

			speed = math.sqrt(vel[1] ^ 2 + vel[2] ^ 2)
			engine.set_velocity_2d(id, vel[1], vel[2])

			local idle = 1
			if speed < idle then
				set_state(id, GLOBALS.ACTIONS.Idle)
			end
		end
	end
end

-- Called once per frame, after all physics substeps have run
function after_physics(dt)
	apply_drag_to_rigids(dt)
end
