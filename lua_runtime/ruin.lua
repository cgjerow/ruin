local ruin = {}

local function ENGINE__update(dt)
  -- called from Rust
  -- invoke the ruin.update function
  ruin.update(dt)
end

function ruin.whatever()

end

function ruin.update(dt)
  -- script update function call
end

return ruin
