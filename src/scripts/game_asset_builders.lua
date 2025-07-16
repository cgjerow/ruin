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

function ControllerBuilder()
	local config = {
		keys = {},
		state = {},
	}

	function config:update(key, is_pressed, mouse_loc)
		local action = self.keys[key]
		if action then
			self.state[action] = { is_pressed = is_pressed, mouse_loc = { x = mouse_loc[1], y = mouse_loc[2] } }
		end
	end

	function config:get_state(action)
		return self.state[action]
	end

	function config:is_pressed(action)
		return self.state[action] and self.state[action].is_pressed
	end

	local builder = {}

	function builder:key(key_str, action_str)
		config.keys[key_str] = action_str
		config.state[action_str] = false -- initialize
		return builder
	end

	function builder:build()
		return config
	end

	return builder
end

function ElementBuilder()
	local element = {
		x = -1000,
		y = -1000,
		height = 1,
		width = 1,
		health = 0,
		state = "Idle",
		base_speed = 20,
		animations = {},
		masks = {},
		layers = {},
	}

	local builder = {}

	function builder:add_mask(mask)
		element.masks[mask] = true
		return builder
	end

	function builder:add_layer(layer)
		element.layers[layer] = true
		return builder
	end

	function builder:add_animation(action, animation)
		element.animations[action] = animation
		return builder
	end

	function builder:size(width, height)
		element.width = width
		element.height = height
		return builder
	end

	function builder:position(x, y)
		element.x = x
		element.y = y
		return builder
	end

	function builder:health(h)
		element.health = h
		return builder
	end

	function builder:build()
		return element
	end

	return builder
end
