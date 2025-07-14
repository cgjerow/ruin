---@diagnostic disable: unused-function, lowercase-global
---
require("main_character")
local pretty_print = require("pretty_print")
require("ghost")
require("dummy")
require("game_asset_builders")

pretty_print(DEATH)

STATE = {
	player = -1,
	characters = {},
	controller = ControllerBuilder()
		:key("W", "Jump")
		:key("S", "Duck")
		:key("A", "MoveLeft")
		:key("D", "MoveRight")
		:build(),
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

function update(dt)
	local speed = 20.0
	local dx, dy = 0, 0

	if STATE.controller:get_state("Jump") then
		dy = dy + 1
	end
	if STATE.controller:get_state("Duck") then
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
		engine.add_velocity(STATE.player, dx * speed, dy * speed, dt)
	end
end

function draw() end

function getState()
	return STATE
end
