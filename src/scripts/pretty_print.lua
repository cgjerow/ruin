local function pretty_print(tbl, indent)
	indent = indent or 0
	local indent_str = string.rep("  ", indent)

	if type(tbl) ~= "table" then
		print(indent_str .. tostring(tbl))
		return
	end

	print(indent_str .. "{")
	for k, v in pairs(tbl) do
		local key_str = tostring(k)
		if type(v) == "table" then
			io.write(indent_str .. "  " .. key_str .. " = ")
			pretty_print(v, indent + 1)
		else
			print(indent_str .. "  " .. key_str .. " = " .. tostring(v))
		end
	end
	print(indent_str .. "}")
end

return pretty_print
