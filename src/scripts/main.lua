---@diagnostic disable: unused-function, lowercase-global
---@diagnostic disable-next-line: unused-local
PRETTY_PRINT = require("pretty_print")
local game_math = require("game_math")
local collisions = require("systems.collisions")
local physics = require("systems.physics")
require("game_asset_builders")

-- Game Elements
local summon_death = require("characters.death")
local skelly = require("characters.skelly")
local new_fence = require("environment.fence")
local new_brick_tile = require("environment.brick_ground")

math.randomseed(os.time())

CONFIG = {
	dead = false,
	input_enabled = true,
	input_disable_time = 0,
	speed = 15.0,
	player_id = -1,
	entities = {},
	controller = ControllerBuilder()
			:key("SPACE", "Dash")
			:key("W", "Up")
			:key("S", "Down")
			:key("A", "MoveLeft")
			:key("D", "MoveRight")
			:build(),
}

WORLD = {
	game_over = false,
	player = { id = -1 },
	skellies = {},
	-- this state is more nuanced then the action state which is used for animations by the engine
	activity_state = {},
	activity_cooldown = {},
	-- this should affect the layers/masks for collision somehow as well
	-- maybe ignoring everything but environment?
	targetability = {},
	kills = 0,
	time = 0,
}
WORLD.player_id = function() return WORLD.player.id end
WORLD.is_game_over = function()
	if WORLD.game_over then
		engine.set_velocity_2d(WORLD.player_id(), 0, 0)
	end
	return WORLD.game_over
end
WORLD.set_game_over = function()
	WORLD.game_over = true
	engine.set_velocity_2d(WORLD.player_id(), 0, 0)
end
WORLD.get_entity = function(id) return CONFIG.entities[id] end
WORLD.set_activity_state = function(id, activity, time, cooldown)
	if not WORLD.activity_state[id] then
		WORLD.activity_state[id] = {}
	end
	if not WORLD.activity_state[id][activity] then
		WORLD.activity_state[id][activity] = {}
	end
	if cooldown and not WORLD.activity_cooldown[id] then
		WORLD.activity_cooldown[id] = {}
	end
	if cooldown and not WORLD.activity_cooldown[id][activity] then
		WORLD.activity_cooldown[id][activity] = {}
	end

	WORLD.activity_state[id][activity].time = time

	if cooldown then
		WORLD.activity_cooldown[id][activity].time = cooldown
	end
end
WORLD.is_activity_going = function(id, activity)
	return WORLD.activity_state[id] and WORLD.activity_state[id][activity] and
			WORLD.activity_state[id][activity].time > 0
end
WORLD.is_activity_done = function(id, activity)
	return not WORLD.activity_state[id] or not WORLD.activity_state[id][activity] or
			WORLD.activity_state[id][activity].time <= 0
end
WORLD.is_off_cooldown = function(id, activity)
	return not WORLD.activity_cooldown[id] or not WORLD.activity_cooldown[id][activity] or
			WORLD.activity_cooldown[id][activity].time <= 0
end
WORLD.tick_activity_state = function(id, activity, dt)
	if WORLD.activity_state[id] and WORLD.activity_state[id][activity] then
		WORLD.activity_state[id][activity].time = WORLD.activity_state[id][activity].time - dt
	end
end
WORLD.tick_cooldown = function(id, activity, dt)
	if WORLD.activity_cooldown[id] and WORLD.activity_cooldown[id][activity] then
		WORLD.activity_cooldown[id][activity].time = WORLD.activity_cooldown[id][activity].time - dt
	end
end


CONTROLLER = {
	start_input_reenable_timer = function(seconds)
		if not CONFIG.input_disable_time or CONFIG.input_disable_time < seconds then
			CONFIG.input_enabled = false
			CONFIG.input_disable_time = seconds
		end
	end
}

ENGINE_HANDLES = {
	create_body = function(entity)
		local result = engine.create_body(entity)
		entity.id = result[1]
		CONFIG.entities[entity.id] = entity
		CONFIG.entities[entity.id].collider = result[2]
		return entity.id
	end,

	set_state = function(id, state)
		if not id == WORLD.player_id() or not WORLD.is_game_over() then
			;
			engine.set_state(id, state)
		end
	end,

	flip_x = function(id, dx)
		if math.abs(dx) > 0.01 then
			engine.flip(id, dx >= 0, false)
		end
	end,

	is_untargetable = function(id)
		return WORLD.targetability[id] and WORLD.targetability[id].duration > 0
	end,

	mark_untargetable = function(id, duration)
		if not WORLD.targetability[id] or WORLD.targetability[id].duration < duration then
			WORLD.targetability[id] = { duration = duration }
			-- local ml = MaskAndLayerBuilder():add_mask(GLOBALS.MASKS_AND_LAYERS.Env):build()
			-- engine.apply_masks_and_layers(WORLD.get_entity(id).collider, ml.masks, ml.layers)
		end
	end,

	tick_targetability = function(dt)
		local to_clear = {}
		for id, targetable in pairs(WORLD.targetability) do
			targetable.duration = targetable.duration - dt
			if (targetable.duration <= 0) then
				local ml = MaskAndLayerBuilder()
				if (id == WORLD.player_id()) then
					ml
							:add_mask(GLOBALS.MASKS_AND_LAYERS.Env)
							:add_mask(GLOBALS.MASKS_AND_LAYERS.Enemy)
							:add_layer(GLOBALS.MASKS_AND_LAYERS.Player)
				else
					ml
							:add_mask(GLOBALS.MASKS_AND_LAYERS.Env)
							:add_mask(GLOBALS.MASKS_AND_LAYERS.Player)
							:add_mask(GLOBALS.MASKS_AND_LAYERS.Enemy)
							:add_layer(GLOBALS.MASKS_AND_LAYERS.Enemy)
				end
				ml = ml:build()
				engine.apply_masks_and_layers(WORLD.get_entity(id).collider, ml.masks, ml.layers)
				table.insert(to_clear, id)
			end
		end
		for _, id in ipairs(to_clear) do
			WORLD.targetability[id] = nil
		end
	end
}


--
--[[ ENGINE CALBACKS ]]
--

-- Called once per frame, after all physics substeps have run
function ENGINE_after_physics(dt)
end

function ENGINE_input_event(input, is_pressed, mouse_position)
	CONFIG.controller:update(string.upper(input), is_pressed, mouse_position, engine.now_ns())
end

function ENGINE_on_collision(cols)
	for _, col in ipairs(cols) do
		collisions.on_each_collision(col)
	end
end

local fps_debug = {
	frame_count = 0,
	time_accum = 0,
	fps = 0,
}

function ENGINE_update(dt)
	-- FPS calculation
	fps_debug.frame_count = fps_debug.frame_count + 1
	fps_debug.time_accum = fps_debug.time_accum + dt

	local id = WORLD.player_id()

	if fps_debug.time_accum >= 1.0 then
		print("UPDATE FPS: ", fps_debug.frame_count)
		fps_debug.frame_count = 0
		fps_debug.time_accum = 0
	end

	if (WORLD.is_game_over()) then return end

	local dx, dy = 0, 0
	ENGINE_HANDLES.tick_targetability(dt)

	WORLD.tick_cooldown(id, GLOBALS.ACTIONS.Dashing, dt)

	local was_dashing = WORLD.is_activity_going(id, GLOBALS.ACTIONS.Dashing)
	if was_dashing then
		WORLD.tick_activity_state(id, GLOBALS.ACTIONS.Dashing, dt)
		if WORLD.is_activity_done(id, GLOBALS.ACTIONS.Dashing) then
			ENGINE_HANDLES.set_state(id, GLOBALS.ACTIONS.Idle)
			engine.set_velocity_2d(id, 0, 0)
		end
	end

	--[[
	if WORLD.activity_state[id].time > 0 then
		WORLD.activity_state[id].time = WORLD.activity_state[id].time - dt

		if WORLD.activity_state[id].activity == GLOBALS.ACTIONS.Dashing then
			if WORLD.activity_state[id].time <= 0 then
				WORLD.activity_state[id].activity = GLOBALS.ACTIONS.Idle
				ENGINE_HANDLES.set_state(id, GLOBALS.ACTIONS.Idle)
				engine.set_velocity_2d(id, 0, 0)
			end
		end
	end
	]]

	--skelly.move(dt)

	--[[ PROCESS INPUT ]]
	--everything after this will only run while input is enabled
	--
	if not CONFIG.input_enabled then
		CONFIG.input_disable_time = CONFIG.input_disable_time - dt
		if CONFIG.input_disable_time <= 0 then
			CONFIG.input_enabled = true
		end
		return
	end

	if CONFIG.controller:is_pressed("Up") then
		dy = dy + 1
	end
	if CONFIG.controller:is_pressed("Down") then
		dy = dy - 1
	end
	if CONFIG.controller:is_pressed("MoveLeft") then
		dx = dx - 1
	end
	if CONFIG.controller:is_pressed("MoveRight") then
		dx = dx + 1
	end

	if CONFIG.controller:is_pressed("Dash") then
		if WORLD.is_off_cooldown(id, GLOBALS.ACTIONS.Dashing) then
			local dash_time = .3
			local dash_speed = 30
			CONTROLLER.start_input_reenable_timer(dash_time)
			ENGINE_HANDLES.mark_untargetable(WORLD.player_id(), .6)
			local x, y = game_math.normalize(dx, dy)

			if not (x == 0 and y == 0) then
				ENGINE_HANDLES.set_state(id, GLOBALS.ACTIONS.Dashing)
				WORLD.set_activity_state(id, GLOBALS.ACTIONS.Dashing, dash_time, .5)
				engine.set_velocity_2d(id, x * dash_speed, y * dash_speed)
				-- leaving this for now as we can implement a "blink" with this if raycasting can prevent
				-- blinking through impassable terrain
				-- engine.apply_move_2d(WORLD.player_id(), x * impulse_strength, y * impulse_strength)
				-- engine.set_velocity_2d(WORLD.player_id(), 0, 0)
			end
		end
	else
		local length = math.sqrt(dx * dx + dy * dy)
		if length > 0 then
			dx = dx / length
			dy = dy / length
			engine.set_velocity_2d(id, dx * CONFIG.speed, dy * CONFIG.speed)
			ENGINE_HANDLES.set_state(id, GLOBALS.ACTIONS.Running)
			ENGINE_HANDLES.flip_x(id, dx)
		else
			engine.set_velocity_2d(id, 0, 0)
			ENGINE_HANDLES.set_state(id, GLOBALS.ACTIONS.Idle)
		end
	end
end

function ENGINE_load()
	engine.create_ui_scene({
		elements = {
			{ position_x = 1, position_y = 2, scale_x = 3, scale_y = 4, width = 5, height = 6, initially_active = true },
			{ position_x = 1, position_y = 2, scale_x = 3, scale_y = 4, width = 5, height = 6, initially_active = false },
		},
		scenes = { {
			elements = {
				{ position_x = 1, position_y = 2, scale_x = 3, scale_y = 4, width = 5, height = 6, initially_active = true },
				{ position_x = 1, position_y = 2, scale_x = 3, scale_y = 4, width = 5, height = 6 },
			},
			scenes = {},
		}
		},
	})

	local death = summon_death(0, 0)
	death.on_collision = "bounce"
	CONFIG.player = death
	WORLD.player.id = ENGINE_HANDLES.create_body(death)

	local build_walls = true
	if build_walls then
		local fence_thickness = 2
		local fence_count_per_side = 25
		local half_length = fence_thickness * fence_count_per_side / 2

		-- Bottom wall: centered on y = -half_length, full width
		local bottom_wall = new_fence(0, -half_length, fence_thickness * fence_count_per_side, fence_thickness)
		bottom_wall.on_player_collision = "block"
		bottom_wall.on_collision = ""
		bottom_wall.id = ENGINE_HANDLES.create_body(bottom_wall)

		-- Top wall: centered on y = half_length, full width
		local top_wall = new_fence(0, half_length, fence_thickness * fence_count_per_side, fence_thickness)
		top_wall.on_player_collision = "block"
		top_wall.on_collision = ""
		top_wall.id = ENGINE_HANDLES.create_body(top_wall)

		-- Left wall: centered on x = -half_length, full height
		local left_wall = new_fence(-half_length, 0, fence_thickness, fence_thickness * fence_count_per_side)
		left_wall.on_player_collision = "block"
		left_wall.on_collision = ""
		left_wall.id = ENGINE_HANDLES.create_body(left_wall)

		-- Right wall: centered on x = half_length, full height
		local right_wall = new_fence(half_length, 0, fence_thickness, fence_thickness * fence_count_per_side)
		right_wall.on_player_collision = "block"
		right_wall.on_collision = ""
		right_wall.id = ENGINE_HANDLES.create_body(right_wall)
	end

	local build_skellys = true
	if build_skellys then
		for _ = 1, 500 do
			local x = math.random(10, 20)
			local y = math.random(10, 20)
			local flip_x = math.random(0, 1)
			local flip_y = math.random(0, 1)
			if flip_x == 1 then
				y = y * -1
			end
			if flip_y == 1 then
				x = x * -1
			end
			local s = skelly.new(x, y)
			-- s.is_pc = true
			s.on_player_collision = "bounce"
			s.on_collision = "bounce"
			s.is_skelly = true
			s.id = ENGINE_HANDLES.create_body(s)
			-- STATE.player = new_id
		end
	end

	return {
		assets = {},
	}
end
