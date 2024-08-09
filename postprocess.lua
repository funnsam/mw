local function to_rel_url(url, depth)
	if string.match(url, "^.+://.*") or string.match(url, "^%./.*") or string.match(url, "^/.*") then
		return url
	else
		local url = string.rep("../", depth) .. url
		if #url ~= 0 then
			return url
		else
			return "./"
		end
	end
end

-- `md_path` is the full path of the markdown file
-- `out_path` is the full path of the output HTML file
-- `depth` is the depth of the file
-- `body` is the compiled HTML
-- `options` is the per-file options
--
-- This global function should return a string of the final HTML
function generate_final_html(md_path, out_path, depth, body, options)
	local title_html = ""
	if options.title ~= nil then
		title_html = string.format("<title>%s</title>", options.title)
	end

	-- Everything in `config.toml` is exported to the global `config`
	local nb_left = ""
	for item = 1, #config.navbar.left do
		local item = config.navbar.left[item]
		if item.md ~= nil and "pages/" .. item.md == path_relative(md_path) then
			nb_left = nb_left .. string.format("<a id=\"active\">%s</a>", item.name)
		else
			nb_left = nb_left .. string.format("<a href=\"%s\">%s</a>", to_rel_url(item.url, depth), item.name)
		end
	end

	local nb_right = ""
	for item = 1, #config.navbar.right do
		local item = config.navbar.right[item]
		if item.md ~= nil and "pages/" .. item.md == path_relative(md_path) then
			nb_right = nb_right .. string.format("<a id=\"active\">%s</a>", item.name)
		else
			nb_right = nb_right .. string.format("<a href=\"%s\">%s</a>", to_rel_url(item.url, depth), item.name)
		end
	end

    if options.search_elsewhere then
        local opts = search_in(project_base .. "/elsewhere", path_parent(out_path), depth)
        -- for i = 1, #opts do
        --     print(opts[i].md_path)
        --     print(opts[i].out_path)
        --     print(opts[i].options.title)
        -- end
    end

	-- KaTeX CSS should be imported for optimal inline math support
	return string.format([[
<!DOCTYPE HTML><html lang="en"><head><meta charset="UTF-8"><link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.10/dist/katex.min.css" crossorigin="anonymous"><link rel="stylesheet" href="%sstyle.css">%s</head><body><nav><span id="nav_left">%s</span><span id="nav_right">%s</span></nav><div id="page_content">%s</div></body></html>
]], string.rep("../", depth), title_html, nb_left, nb_right, body)
end
