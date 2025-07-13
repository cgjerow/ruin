local idle = {
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
	},
	looped = true,
}

GHOST = {
	id = "ghost",
	x = 0,
	y = 0,
	z = 0,
	width = 1,
	height = 1,
	state = "Idle",
	sprite = "ghost.png",
	sprite_sheet_width = 2,
	sprite_sheet_height = 1,
	animations = {
		Idle = idle,
	},
}

return GHOST
