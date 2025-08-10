---@diagnostic disable-next-line: unused-local
local function on_collision(cols)
end

---@diagnostic disable-next-line: unused-local
local function handle_input(input, is_pressed, mouse_position)
end

---@diagnostic disable-next-line: unused-local
function ruin.after_physics(dt)
end

---@diagnostic disable-next-line: unused-local
local function update(dt)
end

local function load()
end

local ruin = {
  on_collision = on_collision,
  handle_input = handle_input,
  update = update,
  load = load,
}

--[[
-- ENGINE_ functions are invoked by the rust engine.
-- These methods should not be leveraged or manipulated directly by Lua scripts.
--]]

function ENGINE_on_collision(cols)
  ruin.on_collision(cols)
end

function ENGINE_input_event(input, is_pressed, mouse_position)
  return ruin.handle_input(input, is_pressed, mouse_position)
end

function ENGINE_load()
  return ruin.load()
end

function ENGINE_after_physics(dt)
  ruin.after_physics(dt)
end

function ENGINE_update(dt)
  ruin.update(dt)
end

--[[
-- ruin table functions are designed to be overriden by Lua scripts.
--]]


-- scripts may override any function defined in ruin global
_G.ruin = ruin

require("scripts.main")
