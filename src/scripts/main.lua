---@diagnostic disable: unused-function, lowercase-global
---
STATE = {
	counter = 0,
	characters = {},
}

require("main_character")
require("ghost")
require("dummy")
require("game_asset_builders")

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
	print("key", key)
	local speed = 30.0 -- or whatever speed you want
	if is_pressed then
		if key == "w" then
			engine.add_velocity(STATE.characters["ghost"], 0, speed)
		elseif key == "s" then
			engine.add_velocity(STATE.characters["ghost"], 0, -speed)
		elseif key == "a" then
			engine.add_velocity(STATE.characters["ghost"], -speed, 0)
		elseif key == "d" then
			engine.add_velocity(STATE.characters["ghost"], speed, 0)
		end
	end
end

function load()
	camera_config = CameraBuilder()
		:mode(Enums.CameraMode.Orthographic2D)
		:speed(20.0)
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

	for x = 0, 0 do
		for y = 0, 0 do
			local character = table.clone(MAIN_CHARACTER)
			character.id = "char_" .. x .. "_" .. y
			character.x = x
			character.y = y
			character.z = -1
			STATE.characters[character.id] = engine.create_character(character)
		end
	end

	--STATE.characters["dummy"] = engine.create_character(DUMMY)
	STATE.characters["ghost"] = engine.create_character(GHOST)

	return {
		assets = {},
	}
end

function update(dt)
	STATE.counter = STATE.counter + 1
end

function draw() end

function getState()
	return STATE
end

function pretty_print(tbl, indent)
	indent = indent or 0
	local indent_str = string.rep("  ", indent)

	if type(tbl) ~= "table" then
		print(indent_str .. tostring(tbl))
		return
	end

	print(indent_str .. "{")
	for k, v in pairs(tbl) do
		local key_str = tostring(k)
		if type(v) == "table" then
			io.write(indent_str .. "  " .. key_str .. " = ")
			pretty_print(v, indent + 1)
		else
			print(indent_str .. "  " .. key_str .. " = " .. tostring(v))
		end
	end
	print(indent_str .. "}")
end
