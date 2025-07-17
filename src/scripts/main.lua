---@diagnostic disable: unused-function, lowercase-global
---@diagnostic disable-next-line: unused-local
local pretty_print = require("pretty_print")
local game_math = require("game_math")
local collisions = require("systems.collisions")
local physics = require("systems.physics")
require("game_asset_builders")

-- Game Elements
local summon_death = require("characters.death")
local skelly = require("characters.skelly")
local new_fence = require("environment.fence")

math.randomseed(os.time())

CONFIG = {
	max_speed = 10,
	skelly_max_speed = 5,
	skelly_speed = 4,
	dead = false,
	friction = 5,
	min_friction = .1,
	input_enabled = true,
	input_disable_time = 0,
	run_force = 200.0,
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
	-- this should affect the layers/masks for collision somehow as well
	-- maybe ignoring everything but environment?
	targetability = {},
	kills = 0,
	time = 0,
}
WORLD.player_id = function() return WORLD.player.id end
WORLD.is_game_over = function() return WORLD.game_over end
WORLD.set_game_over = function() WORLD.game_over = true end


CONTROLLER = {
	start_input_reenable_timer = function(seconds)
		if not CONFIG.input_disable_time or CONFIG.input_disable_time < seconds then
			CONFIG.input_enabled = false
			CONFIG.input_disable_time = seconds
		end
	end
}

ENGINE_HANDLES = {
	set_state = function(id, state)
		if not id == WORLD.player_id() or not WORLD.is_game_over() then
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
			local ml = MaskAndLayerBuilder():add_mask(GLOBALS.MASKS_AND_LAYERS.Env):build()
			engine.apply_masks_and_layers(id, ml.masks, ml.layers)
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
				engine.apply_masks_and_layers(id, ml.masks, ml.layers)
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
	physics.apply_drag_to_rigids(dt)
end

function ENGINE_input_event(input, is_pressed, mouse_position)
	CONFIG.controller:update(string.upper(input), is_pressed, mouse_position, engine.now_ns())
end

function ENGINE_on_collision(cols)
	for _, col in ipairs(cols) do
		collisions.on_each_collision(col)
	end
end

function ENGINE_update(dt)
	if (WORLD.is_game_over()) then return end

	local dx, dy = 0, 0
	ENGINE_HANDLES.tick_targetability(dt)

	skelly.move()

	if not CONFIG.input_enabled then
		CONFIG.input_disable_time = CONFIG.input_disable_time - dt
		if CONFIG.input_disable_time <= 0 then
			CONFIG.input_enabled = true
		end
		return
	end

	if CONFIG.controller:is_pressed("Dash") then
		CONTROLLER.start_input_reenable_timer(0.5)
		ENGINE_HANDLES.mark_untargetable(WORLD.player_id(), .6)
		local v = engine.get_velocity_2d(WORLD.player_id())
		local x, y = game_math.normalize(v[1], v[2])

		if not (x == 0 and y == 0) then
			local impulse_strength = 200
			engine.apply_impulse_2d(WORLD.player_id(), x * impulse_strength, y * impulse_strength)
		end
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

	-- Normalize direction vector if needed
	local length = math.sqrt(dx * dx + dy * dy)
	if length > 0 then
		dx = dx / length
		dy = dy / length
		engine.apply_force_2d(WORLD.player_id(), dx * CONFIG.run_force, dy * CONFIG.run_force)
		ENGINE_HANDLES.set_state(WORLD.player_id(), GLOBALS.ACTIONS.Running)
		ENGINE_HANDLES.flip_x(WORLD.player_id(), dx)
	end
end

function ENGINE_load()
	local death = summon_death(0, 0)
	death.on_collision = "bounce"
	CONFIG.player = death
	WORLD.player.id = engine.create_body(death)
	death.id = WORLD.player_id()
	CONFIG.entities[WORLD.player_id()] = death

	local build_walls = true
	if build_walls then
		for i = 0, 50 do
			if i % 2 == 0 then
				goto continue
			end
			local fence = new_fence(i - 25, -25)
			fence.on_player_collision = "block"
			fence.on_collision = ""
			fence.id = engine.create_body(fence)
			CONFIG.entities[fence.id] = fence

			fence = new_fence(i - 25, 25)
			fence.on_player_collision = "block"
			fence.on_collision = ""
			fence.id = engine.create_body(fence)
			CONFIG.entities[fence.id] = fence

			fence = new_fence(-25, i - 25)
			fence.on_player_collision = "block"
			fence.on_collision = ""
			fence.id = engine.create_body(fence)
			CONFIG.entities[fence.id] = fence

			fence = new_fence(25, i - 25)
			fence.on_player_collision = "block"
			fence.on_collision = ""
			fence.id = engine.create_body(fence)
			CONFIG.entities[fence.id] = fence
			::continue::
		end
	end

	local build_skellys = true
	if build_skellys then
		for _ = 1, 10 do
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
			s.id = engine.create_body(s)
			s.on_player_collision = "bounce"
			s.on_collision = "bounce"
			s.is_skelly = true
			CONFIG.entities[s.id] = s
			-- STATE.player = new_id
		end
	end

	return {
		assets = {},
	}
end
