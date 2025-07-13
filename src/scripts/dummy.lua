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

DUMMY = {
	id = "dummy",
	x = 0,
	y = 0,
	z = 0,
	width = 1,
	height = 1,
	state = "Idle",
	sprite = "lpc_entry/png/combat_dummy/BODY_animation.png",
	sprite_sheet_width = 8,
	sprite_sheet_height = 1,
	animations = {
		Idle = idle,
	},
}

return DUMMY
