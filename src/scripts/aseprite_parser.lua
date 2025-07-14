local json = require("json") -- adapt to your preferred JSON library

local function load_aseprite_animation(animation_name, path, json_file)
	local json_path = "src/assets/" .. path .. json_file
	------------------------------------------------------------------ I/O
	local fh, io_err = io.open(json_path, "r")
	if not fh then
		return nil, ("cannot open “%s”: %s"):format(json_path, io_err)
	end
	local raw = fh:read("*a")
	fh:close()

	---------------------------------------------------------------- decode
	local data, pos, decode_err = json.decode(raw)
	if not data then
		return nil, ("JSON error in “%s” at byte %d: %s"):format(json_path, pos or 0, decode_err)
	end

	local frames = data.frames
	if not (frames and #frames > 0) then
		return nil, ("no frames found in “%s”"):format(json_path)
	end

	---------------------------------------------------------------- sheet geometry
	local fw, fh = frames[1].frame.w, frames[1].frame.h
	local sheet_cols = data.meta.size.w // fw -- integer division
	local sheet_rows = data.meta.size.h // fh

	---------------------------------------------------------------- build anim table
	local anim = {
		sprite = path .. data.meta.image, -- e.g. "death_idle.png"
		sprite_sheet_width = sheet_cols,
		sprite_sheet_height = sheet_rows,
		frames = {},
		looped = true, -- default; may be overridden below
	}

	for _, fr in ipairs(frames) do
		local gx = fr.frame.x // fw -- column index in sheet grid
		local gy = fr.frame.y // fh -- row    index
		anim.frames[#anim.frames + 1] = {
			x = gx,
			y = gy,
			duration = (fr.duration or 100) / 1000, -- ms → seconds
		}
	end

	---------------------------------------------------------------- frameTags → looped?
	if data.meta.frameTags and #data.meta.frameTags > 0 then
		for _, tag in ipairs(data.meta.frameTags) do
			if tag.name == animation_name then
				-- Aseprite directions: "forward", "reverse", "pingpong"
				anim.looped = (tag.direction == "forward")
				break
			end
		end
	end

	return anim
end

return load_aseprite_animation
