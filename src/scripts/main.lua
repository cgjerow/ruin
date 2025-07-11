---@diagnostic disable: unused-function, lowercase-global
---
STATE = {
	counter = 0,
}

require("main_character")

function load()
	print("LUA: Load Game")

	engine.create_character(MAIN_CHARACTER)

	return {
		assets = {
			"mittens-goblin-art.png",
			"happy_tree.png",
			"braid-sprite-sheet.webp",
		},
	}
end

function update(dt)
	STATE.counter = STATE.counter + 1
	print("LUA: Update: ", STATE.counter, engine.get_window_size())
end

function draw()
	print("LUA Draw")
end

function getState()
	return STATE
end
