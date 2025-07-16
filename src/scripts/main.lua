---@diagnostic disable: unused-function, lowercase-global
require("game_asset_builders")
local pretty_print = require("pretty_print")

-- Game Elements
local summon_death = require("characters.death")
local new_skelly = require("characters.skelly")
local new_fence = require("environment.fence")

math.randomseed(os.time())

STATE = {
	dead = false,
	input_enabled = true,
	input_disable_time = 0,
	speed = 500.0,
	player_id = -1,
	entities = {},
	untargetable = {},
	controller = ControllerBuilder():key("SPACE", "Dash"):key("W", "Up"):key("S", "Down"):key("A", "MoveLeft"):key("D", "MoveRight"):build(),
}

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
	--pretty_print(death)
	death.on_player_collision = "bounce"
	death.on_collision = "bounce"
	STATE.player = death
	STATE.player_id = engine.create_element(death)
	death.id = STATE.player_id
	STATE.entities[STATE.player_id] = death

	for i = 0, 50 do
		if i % 2 == 0 then
			goto continue
		end
		local fence = new_fence(i - 25, -25)
		fence.on_player_collision = "block"
		fence.on_collision = ""
		fence.id = engine.create_element(fence)
		STATE.entities[fence.id] = fence

		fence = new_fence(i - 25, 25)
		fence.on_player_collision = "block"
		fence.on_collision = ""
		fence.id = engine.create_element(fence)
		STATE.entities[fence.id] = fence

		fence = new_fence(-25, i - 25)
		fence.on_player_collision = "block"
		fence.on_collision = ""
		fence.id =  engine.create_element(fence)
		STATE.entities[fence.id] = fence

		fence = new_fence(25, i - 25)
		fence.on_player_collision = "block"
		fence.on_collision = ""
		fence.id = engine.create_element(fence)
		STATE.entities[fence.id] = fence
		::continue::
	end

	for _i = 1, 10 do
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
		s.id = engine.create_element(s)
		s.on_player_collision = "bounce"
		s.on_collision = "bounce"
		STATE.entities[s.id] = s
		-- STATE.player = new_id
	end

	return {
		assets = {},
	}
end

function on_entity_idle(entities)
	-- if we need to, update state
	for _, entity in pairs(entities) do
		set_state(entity, GLOBALS.ACTIONS.Idle)
	end
end

function on_collision(collisions)
	local acceleration = 1.0
	local bounce_speed = 1.0

	for _i, collision in ipairs(collisions) do
		local a_id = collision.entity_a
		local b_id = collision.entity_b

		local normal_x = collision.normal[1]
		local normal_y = collision.normal[2]
		local length = math.sqrt(normal_x ^ 2 + normal_y ^ 2)

		if length == 0 then
			normal_x = 1
			normal_y = 0
		else
			normal_x = normal_x / length
			normal_y = normal_y / length
		end

		local pos_a = collision.next_pos_a
		local pos_b = collision.next_pos_b
		local size_a = collision.a_size
		local size_b = collision.b_size

		local delta_x = pos_a[1] - pos_b[1]
		local delta_y = pos_a[2] - pos_b[2]

		local half_a_x = size_a[1] * 0.5
		local half_a_y = size_a[2] * 0.5
		local half_b_x = size_b[1] * 0.5
		local half_b_y = size_b[2] * 0.5

		local proj_a = math.abs(half_a_x * normal_x) + math.abs(half_a_y * normal_y)
		local proj_b = math.abs(half_b_x * normal_x) + math.abs(half_b_y * normal_y)

		local delta_proj = delta_x * normal_x + delta_y * normal_y

		local penetration = proj_a + proj_b - math.abs(delta_proj)

		if penetration > 0 then
			local sep_x = normal_x * penetration
			local sep_y = normal_y * penetration

			if a_id == STATE.player_id or b_id == STATE.player_id then
				if a_id == STATE.player_id then
					if STATE.entities[b_id].on_player_collision == "block" then
						local penetration_block = proj_a - math.abs(delta_proj)
						if penetration_block > 0 then
							-- Separation vector for blocking, no bounce velocity
							sep_x = normal_x * penetration_block * 1.2
							sep_y = normal_y * penetration_block * 1.2

							-- Stop player's velocity along collision normal
							set_state(STATE.player_id, GLOBALS.ACTIONS.Idle)
							engine.redirect(a_id, 0, 0, sep_x, sep_y, 0)
						end
					end
					if STATE.entities[b_id].on_player_collision == "bounce" then
						engine.redirect(
							a_id,
							normal_x * bounce_speed,
							normal_y * bounce_speed,
							sep_x,
							sep_y,
							acceleration
						)
						set_state(STATE.player_id, GLOBALS.ACTIONS.Idle)
						STATE.input_enabled = false
						start_input_reenable_timer(0.3)
					end
					if STATE.entities[b_id].on_collision == "bounce" then
						engine.redirect(
							b_id,
							-normal_x * bounce_speed,
							-normal_y * bounce_speed,
							-sep_x,
							-sep_y,
							acceleration
						)
						set_state(b_id, GLOBALS.ACTIONS.Idle)
					end
				end

				if b_id == STATE.player_id then
					if STATE.entities[a_id].on_player_collision == "bounce" then
						engine.redirect(
							b_id,
							-normal_x * bounce_speed,
							-normal_y * bounce_speed,
							-sep_x,
							-sep_y,
							acceleration
						)
						set_state(STATE.player_id, GLOBALS.ACTIONS.Idle)
						STATE.input_enabled = false
						start_input_reenable_timer(0.3)
					end
					if STATE.entities[a_id].on_collision == "bounce" then
						engine.redirect(
							a_id,
							normal_x * bounce_speed,
							normal_y * bounce_speed,
							sep_x,
							sep_y,
							acceleration
						)
						set_state(a_id, GLOBALS.ACTIONS.Idle)
					end
				end
				if (STATE.entities[a_id] and STATE.entities[a_id].layers[GLOBALS.MASKS_AND_LAYERS.Enemy]) or
						(STATE.entities[b_id] and STATE.entities[b_id].layers[GLOBALS.MASKS_AND_LAYERS.Enemy]) then
					if not ((STATE.untargetable[a_id] and STATE.untargetable[a_id] > 0) or
								(STATE.untargetable[b_id] and STATE.untargetable[b_id] > 0)) then
						STATE.untargetable[STATE.player_id] = 1
						print("DAMAGE")
						local dead = engine.damage(STATE.player_id, 2)
						print("surely hes dead", dead)
						if dead == true then
							set_state(STATE.player_id, GLOBALS.ACTIONS.Dying)
							STATE.dead = true
							STATE.input_enabled = false
							start_input_reenable_timer(100)
						end
					end
				end
			end
		end
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
		local dash = STATE.controller:get_state("Dash")
		pretty_print(dash.mouse_loc)
		engine.redirect_to(
			STATE.player_id,
			dash.mouse_loc.x,
			dash.mouse_loc.y,
			1000,
			0
		)
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
		engine.add_acceleration(STATE.player_id, dx * STATE.speed, dy * STATE.speed)
		set_state(STATE.player_id, GLOBALS.ACTIONS.Running)
		if math.abs(dx) > 0.01 then
			engine.flip(STATE.player_id, dx >= 0, false)
		end
	end
end

function draw() end

function getState()
	return STATE
end
