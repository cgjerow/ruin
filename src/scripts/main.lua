---@diagnostic disable: unused-function, lowercase-global
---
require("main_character")
local pretty_print = require("pretty_print")
require("game_asset_builders")
local new_skelly = require("skelly")
local new_fence = require("fence")

math.randomseed(os.time())

STATE = {
	input_enabled = true,
	input_disable_time = 0,
	speed = 500.0,
	player = -1,
	entities = {},
	controller = ControllerBuilder():key("W", "Up"):key("S", "Down"):key("A", "MoveLeft"):key("D", "MoveRight"):build(),
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
	STATE.input_disable_time = seconds
end

function keyboard_event(key, is_pressed)
	key = string.upper(key)
	STATE.controller:update(key, is_pressed)
end

function load()
	camera_config = CameraBuilder()
		:mode(Enums.CameraMode.Orthographic2D)
		:speed(20.0)
		:locked(true)
		:key("W", "MoveForward")
		:key("S", "MoveBackward")
		:key("A", "MoveLeft")
		:key("D", "MoveRight")
		:key("Q", "RollLeft")
		:key("E", "RollRight")
		:key("Up", "PitchUp")
		:key("Down", "PitchDown")
		:key("Left", "YawLeft")
		:key("Right", "YawRight")
		:build()

	engine.configure_camera(camera_config)
	STATE.player = engine.create_character(DEATH)
	STATE.entities[STATE.player] = {
		id = STATE.player,
		class = "enemy",
		on_player_collision = "bounce",
		on_collision = "bounce",
	}

	for i = 0, 50 do
		if i % 2 == 0 then
			goto continue
		end
		local fence = new_fence(i - 25, -25)
		local new_id = engine.create_character(fence)
		STATE.entities[new_id] = {
			id = new_id,
			class = "wall",
			on_player_collision = "block",
			on_collision = "",
		}

		fence = new_fence(i - 25, 25)
		new_id = engine.create_character(fence)
		STATE.entities[new_id] = {
			id = new_id,
			class = "wall",
			on_player_collision = "block",
			on_collision = "",
		}

		fence = new_fence(-25, i - 25)
		new_id = engine.create_character(fence)
		STATE.entities[new_id] = {
			id = new_id,
			class = "wall",
			on_player_collision = "block",
			on_collision = "",
		}

		fence = new_fence(25, i - 25)
		new_id = engine.create_character(fence)
		STATE.entities[new_id] = {
			id = new_id,
			class = "wall",
			on_player_collision = "block",
			on_collision = "",
		}
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
		local new_id = engine.create_character(s)
		STATE.entities[new_id] = {
			id = new_id,
			class = "enemy",
			on_player_collision = "bounce",
			on_collision = "bounce",
		}
		-- STATE.player = new_id
	end

	return {
		assets = {},
	}
end

function on_entity_idle(entities)
	-- if we need to, update state
	for _, entity in pairs(entities) do
		engine.set_state(entity, "Idle")
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

			if a_id == STATE.player or b_id == STATE.player then
				if a_id == STATE.player then
					if STATE.entities[b_id].on_player_collision == "block" then
						local penetration_block = proj_a - math.abs(delta_proj)
						if penetration_block > 0 then
							-- Separation vector for blocking, no bounce velocity
							sep_x = normal_x * penetration_block * 1.2
							sep_y = normal_y * penetration_block * 1.2

							-- Stop player's velocity along collision normal
							engine.set_state(STATE.player, "Idle")
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
						engine.set_state(STATE.player, "Idle")
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
						engine.set_state(b_id, "Idle")
					end
				end

				if b_id == STATE.player then
					if STATE.entities[a_id].on_player_collision == "bounce" then
						engine.redirect(
							b_id,
							-normal_x * bounce_speed,
							-normal_y * bounce_speed,
							-sep_x,
							-sep_y,
							acceleration
						)
						engine.set_state(STATE.player, "Idle")
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
						engine.set_state(a_id, "Idle")
					end
				end
			end
		end
	end
end

function update(dt)
	local dx, dy = 0, 0
	if not STATE.input_enabled then
		STATE.input_disable_time = STATE.input_disable_time - dt
		if STATE.input_disable_time <= 0 then
			STATE.input_enabled = true
		end
		return
	end

	if STATE.controller:get_state("Up") then
		dy = dy + 1
	end
	if STATE.controller:get_state("Down") then
		dy = dy - 1
	end
	if STATE.controller:get_state("MoveLeft") then
		dx = dx - 1
	end
	if STATE.controller:get_state("MoveRight") then
		dx = dx + 1
	end

	-- Normalize direction vector if needed
	local length = math.sqrt(dx * dx + dy * dy)
	if length > 0 then
		dx = dx / length
		dy = dy / length
		engine.add_acceleration(STATE.player, dx * STATE.speed, dy * STATE.speed)
		engine.set_state(STATE.player, "Running")
		if math.abs(dx) > 0.01 then
			engine.flip(STATE.player, dx >= 0, false)
		end
	end
end

function draw() end

function getState()
	return STATE
end
