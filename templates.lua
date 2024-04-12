local function get_parent_path(path)
	return string.match(path, "^(.+)/")
end

local function get_rel_path(path, depth)
	local _, i = string.gsub(path, "([^/]+)/?", "")
	local pop = i - depth + 1
	for j = 0, pop do
		path = get_parent_path(path)
		print(path)
	end

	return path
end

function generate_final_html(path, depth, body, options)
	print(get_rel_path(path, depth))
	local title_html = ""
	if options.title ~= nil then
		title_html = string.format("<title>%s</title>", options.title)
	end

	local nb_left = ""
	for item = 1, #navbar.left do
		local item = navbar.left[item]
		local active = ""
		if item.md == get_rel_path(path, depth) then
			active = " id=\"active\""
		end
		nb_left = nb_left .. string.format("<a href=\"%s\"%s>%s</a>", item.url, active, item.name)
	end

	local nb_right = ""
	for item = 1, #navbar.right do
		local item = navbar.right[item]
		local active = ""
		if item.name == options.title then
			active = " id=\"active\""
		end
		nb_right = nb_right .. string.format("<a href=\"%s\"%s>%s</a>", item.url, active, item.name)
	end

	return string.format([[
<!DOCTYPE HTML><html lang="en"><head><meta charset="UTF-8"><link rel="stylesheet" href="%sstyle.css">%s</head><body><nav><span id="nav_left">%s</span><span id="nav_right">%s</span></nav><div id="page_content">%s</div></body></html>
]], string.rep("../", depth), title_html, nb_left, nb_right, body)
end
