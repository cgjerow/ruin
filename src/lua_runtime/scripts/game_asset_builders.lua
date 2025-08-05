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

function PhysicsBodyBuilder()
	local body = {
		type = GLOBALS.PHYSICS_BODIES.Rigid,
		x = -1000,
		y = -1000,
		collision_box = {
			enabled = true,
			offset_x = 0,
			offset_y = 0,
			size_modifier_x = 1,
			size_modifier_y = 1,
		},
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
		body.masks[mask] = true
		return builder
	end

	function builder:body_type(type)
		body.type = type
		return builder
	end

	function builder:add_layer(layer)
		body.layers[layer] = true
		return builder
	end

	function builder:add_animation(action, animation)
		body.animations[action] = animation
		return builder
	end

	function builder:size(width, height)
		body.width = width
		body.height = height
		return builder
	end

	function builder:position(x, y)
		body.x = x
		body.y = y
		return builder
	end

	function builder:collider_offset(x, y)
		body.collision_box.offset_x = x
		body.collision_box.offset_y = y
		return builder
	end

	function builder:collider_size_modifier(x, y)
		body.collision_box.size_modifier_x = x
		body.collision_box.size_modifier_y = y
		return builder
	end

	function builder:health(h)
		body.health = h
		return builder
	end

	function builder:build()
		return body
	end

	return builder
end

function MaskAndLayerBuilder()
	local result = {
		masks = {},
		layers = {},
	}

	local builder = {}

	function builder:add_mask(mask)
		result.masks[mask] = true
		return builder
	end

	function builder:add_layer(layer)
		result.layers[layer] = true
		return builder
	end

	function builder:build()
		return result
	end

	return builder
end

function CanvasSceneBuilder()
	local body = {
		x = -1000,
		y = -1000,
		height = 1,
		width = 1,
		state = "Idle",
		animations = {},
	}

	local builder = {}

	function builder:add_animation(action, animation)
		body.animations[action] = animation
		return builder
	end

	function builder:size(width, height)
		body.width = width
		body.height = height
		return builder
	end

	function builder:position(x, y)
		body.x = x
		body.y = y
		return builder
	end

	function builder:build()
		return body
	end

	return builder
end
