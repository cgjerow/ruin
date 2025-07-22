local json = require("json") -- adapt to your preferred JSON library

local function AnimationBuilder()
	local anim = {
		sprite = "",
		tile_height = 32,
		tile_width = 32,
		sprite_sheet_height = 0,
		sprite_sheet_width = 0,
		frames = {},
		hitboxes = {},
		hurtboxes = {},
		looped = true,
		is_transparent = false,
	}

	local builder = {}

	function builder:set_sprite(s)
		anim.sprite = s
		return builder
	end

	function builder:set_layout(w, h)
		anim.sprite_sheet_width = w
		anim.sprite_sheet_height = h
		return builder
	end

	function builder:loop(b)
		anim.looped = b
		return builder
	end

	function builder:add_frame(f)
		anim.frames[#anim.frames + 1] = f
		return builder
	end

	function builder:add_hb(f, t, hb, masks, layers)
		t = t .. "es"
		if not anim[t][f] then
			anim[t][f] = {}
		end

		local flipped_y = anim.tile_height - (hb.y + hb.h)
		anim[t][f][#anim[t][f] + 1] = {
			center_x = hb.x + (hb.w / 2),
			center_y = flipped_y + (hb.h / 2),
			width = hb.w,
			height = hb.h,
			masks,
			layers,
		}
		PRETTY_PRINT(anim[t][f])
	end

	function builder:transparency(b)
		anim.is_transparent = b
		return builder
	end

	function builder:build()
		return anim
	end

	return builder
end

local function load_aseprite_animation(animation_name, path, json_file, with_transparency)
	local json_path = "src/assets/" .. path .. json_file
	local file, io_err = io.open(json_path, "r")
	if not file then
		return nil, ("cannot open “%s”: %s"):format(json_path, io_err)
	end
	local raw = file:read("*a")
	file:close()

	local builder = AnimationBuilder()

	local data, pos, decode_err = json.decode(raw)
	if not data then
		return nil, ("JSON error in “%s” at byte %d: %s"):format(json_path, pos or 0, decode_err)
	end

	local frames = data.frames
	if not (frames and #frames > 0) then
		return nil, ("no frames found in “%s”"):format(json_path)
	end

	local looped = true
	if data.meta.frameTags and #data.meta.frameTags > 0 then
		for _, tag in ipairs(data.meta.frameTags) do
			if tag.name == animation_name then
				-- Aseprite directions: "forward", "reverse", "pingpong"
				looped = (tag.direction == "forward")
				break
			end
		end
	end

	local fw, fh = frames[1].frame.w, frames[1].frame.h
	builder
			:set_sprite(path .. data.meta.image)
			:set_layout(data.meta.size.w // fw, data.meta.size.h // fh)
			:loop(looped)
			:transparency(with_transparency == true)

	for _, fr in ipairs(frames) do
		local gx = fr.frame.x // fw -- column index in sheet grid
		local gy = fr.frame.y // fh -- row    index
		builder:add_frame(
			{
				x = gx,
				y = gy,
				duration = (fr.duration or 100) / 1000, -- ms → seconds
			}
		)
	end

	local slices = data.meta.slices
	if slices and #slices > 0 then
		for _, s in ipairs(slices) do
			if s.name == "hitbox" or s.name == "hurtbox" then
				local hb = json.decode(s.data)
				if hb.frames and #hb.frames > 0 then
					for _, f in ipairs(hb.frames) do
						builder:add_hb(f, s.name, s.keys[1].bounds, hb.masks, hb.layers)
					end
				end
			end
		end
	end

	return builder:build()
end

return load_aseprite_animation
