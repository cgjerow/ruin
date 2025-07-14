Enums = {
	CameraMode = {
		Orthographic2D = "Orthographic2D",
		Perspective3D = "Perspective3D",
		Universal = "Universal",
	},
}

function CameraBuilder()
	local config = {
		mode = "Orthographic2D",
		speed = 10.0,
		locked = true,
		keys = {},
	}

	local builder = {}

	function builder:locked(b)
		config.locked = b
		return builder
	end

	function builder:mode(m)
		config.mode = m
		return builder
	end

	function builder:speed(s)
		config.speed = s
		return builder
	end

	function builder:key(key_str, action_str)
		table.insert(config.keys, { key = key_str, action = action_str })
		return builder
	end

	function builder:build()
		return config
	end

	return builder
end
