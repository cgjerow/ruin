---@diagnostic disable: unused-function, lowercase-global
---
require("main_character")
local pretty_print = require("pretty_print")
require("ghost")
require("dummy")
require("game_asset_builders")

pretty_print(DEATH)

STATE = {
	input_enabled = true,
	input_disable_time = 0,
	speed = 1000.0,
	player = -1,
	characters = {},
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
	STATE.player = engine.create_character(DEATH)

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
	local bounce_speed = 100.0 -- adjust to your desired bounce magnitude
	local push = 1.0 -- tweak this constant

	for _i, collision in ipairs(collisions) do
		local a_id = collision.entity_a
		local b_id = collision.entity_b

		-- Compute collision normal (from B to A)
		local normal_x = collision.normal[1]
		local normal_y = collision.normal[2]
		local length = math.sqrt(normal_x ^ 2 + normal_y ^ 2)

		local sep_x = normal_x * push
		local sep_y = normal_y * push

		if length == 0 then
			-- Entities perfectly overlapping, pick arbitrary normal
			normal_x = 1
			normal_y = 0
		else
			normal_x = normal_x / length
			normal_y = normal_y / length
		end

		print(a_id, b_id, STATE.player)
		if a_id == STATE.player then
			-- Redirect entity A with velocity bouncing away along normal
			print("one")
			engine.redirect(a_id, normal_x * bounce_speed, normal_y * bounce_speed, sep_x, sep_y)
		end
		if b_id == STATE.player then
			print("two")
			-- Optionally redirect entity B if dynamic, bounce opposite normal
			engine.redirect(b_id, -normal_x * bounce_speed, -normal_y * bounce_speed, -sep_x, -sep_y)
		end
		engine.set_state(STATE.player, "Idle")
		STATE.input_enabled = false
		STATE.input_enabled = false
		-- Optionally start a timer to re-enable input after delay
		start_input_reenable_timer(0.3) -- re-enable after 1 second
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
