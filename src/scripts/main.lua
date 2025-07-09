---@diagnostic disable: unused-function, lowercase-global
---
STATE = {
	counter = 0,
}

function load()
	print("LUA: Load Game")
	return {
		assets = {
			"mittens-goblin-art.png",
			"happy_tree.png",
		},
	}
end

function update(dt)
	STATE.counter = STATE.counter + 1
	print("LUA: Update {}", STATE.counter)
end

function draw()
	print("LUA Draw")
end

function getState()
	return STATE
end
