function generate_final_html(path, body, options)
	local title_html = ""
	if options["title"] ~= nil then
		title_html = string.format("<title>%s</title>", options["title"])
	end

	return string.format([[
<!DOCTYPE HTML><html lang="en"><head><meta charset="UTF-8"><link rel="stylesheet" href="/style.css">%s</head><body>%s</body></html>
]], title_html, body)
end
