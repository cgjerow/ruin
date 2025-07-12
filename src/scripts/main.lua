---@diagnostic disable: unused-function, lowercase-global
---
STATE = {
	counter = 0,
}

require("main_character")
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

function load()
	-- print("LUA: Load Game")

	engine.create_character(MAIN_CHARACTER)
	camera_config = CameraBuilder()
		:mode(Enums.CameraMode.Universal)
		:speed(10.0)
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

	for x = 0, 99 do
		for y = 0, 9 do
			local character = table.clone(MAIN_CHARACTER)
			character.id = "char_" .. x .. "_" .. y
			character.x = x
			character.y = y
			engine.create_character(character)
		end
	end

	return {
		assets = {},
	}
end

function update(dt)
	STATE.counter = STATE.counter + 1
	-- print("LUA: Update: ", STATE.counter, engine.get_window_size())
end

function draw()
	-- print("LUA Draw")
end

function getState()
	return STATE
end
