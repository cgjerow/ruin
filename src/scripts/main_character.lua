local running = {
	frames = {
		{
			x = 0,
			y = 0,
			duration = 0.075,
		},
		{
			x = 1,
			y = 0,
			duration = 0.075,
		},
		{
			x = 2,
			y = 0,
			duration = 0.075,
		},
		{
			x = 3,
			y = 0,
			duration = 0.075,
		},
		{
			x = 4,
			y = 0,
			duration = 0.075,
		},
		{
			x = 5,
			y = 0,
			duration = 0.075,
		},
		{
			x = 6,
			y = 0,
			duration = 0.075,
		},
		{
			x = 7,
			y = 0,
			duration = 0.075,
		},
		{
			x = 1,
			y = 1,
			duration = 0.075,
		},
		{
			x = 2,
			y = 1,
			duration = 0.075,
		},
		{
			x = 3,
			y = 1,
			duration = 0.075,
		},
		{
			x = 4,
			y = 1,
			duration = 0.075,
		},
		{
			x = 5,
			y = 1,
			duration = 0.075,
		},
		{
			x = 6,
			y = 1,
			duration = 0.075,
		},
	},
	looped = true,
}

MAIN_CHARACTER = {
	id = "braid",
	x = 0,
	y = 0,
	width = 1,
	height = 1,
	state = "Running",
	sprite = "lpc_entry/png/thrust/BODY_animation.png",
	sprite_sheet_width = 8,
	sprite_sheet_height = 4,
	animations = {
		Running = running,
	},
}

return MAIN_CHARACTER
