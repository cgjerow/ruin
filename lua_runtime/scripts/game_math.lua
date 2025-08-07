local function normalize(x, y)
	local mag = math.sqrt(x ^ 2 + y ^ 2)
	if mag == 0 then
		return 0, 0
	end
	return x / mag, y / mag
end

return { normalize = normalize }
