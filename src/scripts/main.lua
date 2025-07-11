---@diagnostic disable: unused-function, lowercase-global
---
STATE = {
	counter = 0,
}

function load()
	print("LUA: Load Game")

	engine.create_character({
		id = "braid",
		x = 0,
		y = 0,
		width = 1,
		height = 1,
		state = "Running",
		sprite = "braid-sprite-sheet.webp",
		sprite_sheet_width = 7,
		sprite_sheet_height = 4,
		animations = {
			Running = {
				frames = {
					{
						x = 0,
						y = 0,
						duration = 0.2,
					},
					{
						x = 1,
						y = 0,
						duration = 0.2,
					},
					{
						x = 2,
						y = 0,
						duration = 0.2,
					},
					{
						x = 3,
						y = 0,
						duration = 0.2,
					},
					{
						x = 4,
						y = 0,
						duration = 0.2,
					},
					{
						x = 5,
						y = 0,
						duration = 0.2,
					},
					{
						x = 6,
						y = 0,
						duration = 0.2,
					},
					{
						x = 7,
						y = 0,
						duration = 0.2,
					},
					{
						x = 1,
						y = 1,
						duration = 0.2,
					},
					{
						x = 2,
						y = 1,
						duration = 0.2,
					},
					{
						x = 3,
						y = 1,
						duration = 0.2,
					},
					{
						x = 4,
						y = 1,
						duration = 0.2,
					},
					{
						x = 5,
						y = 1,
						duration = 0.2,
					},
					{
						x = 6,
						y = 1,
						duration = 0.2,
					},
				},
				looped = true,
			},
		},
	})

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
