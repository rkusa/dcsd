--[[
	Installation:
	1. move this file into %USERPROFILE%\Saved Games\DCS.openalpha\Scripts\
	   (or create a symbolic link, like:
	    New-Item -Path "M:\Saved Games\DCS.openalpha\Scripts\dcsd.lua" -ItemType HardLink -Value "M:\Desktop\dcsd\dcsd.lua"
	    )
	2. go to DCS installation directory and add the following to Scripts\MissionScripting.lua
	   (before sanitizeModule):
	   		dcsd = {}
			dofile(lfs.writedir().."Scripts/dcsd.lua")
	3. add the following to the mission (in a ONCE -> TIME MORE (1) -> DO SCRIPT trigger):
			dcsd.start(_G)
]]--

-- show LUA errors in ingame message box
-- env.setErrorMessageBoxEnabled(false)

package.path = package.path..";.\\LuaSocket\\?.lua"
package.cpath = package.cpath..";.\\LuaSocket\\?.dll"
local socket = require("socket")
local JSON = loadfile("Scripts\\JSON.lua")()

env.info("initializing dcsd ...")

dcsd = {}

dcsd.receive = function()
	-- env.info("[dcsd] receiving data ...")

	local line = dcsd.udp:receive()
	if line then
		env.info("[dcsd] received: "..line)
		trigger.action.outText(line, 0)
	end
end

dcsd.start = function(mission_env_, host, port)
	dcsd.mission_env = mission_env_

	if host == nil then
		host = "127.0.0.1"
	end

	if port == nil then
		port = 8080
	end

	env.info("starting dcsd ...")
	dcsd.udp = socket.udp()
	-- dcsd.udp:setsockname("*", 0)
	dcsd.udp:setsockname("*", 8081)
	dcsd.udp:settimeout(0)

	local fn = timer.scheduleFunction(function(arg, time)
		dcsd.receive()

		-- return time of next call
		return timer.getTime() + .1
	end, nil, timer.getTime() + .1)

	local eventHandler = {}
	function eventHandler:onEvent(event)
		env.info("[dcsd] event "..event.id)

		socket.try(dcsd.udp:sendto("ev:"..event.id.."\n", host, port))
		--socket.try(dcsd.conn:send(JSON:encode(event):gsub("\n", "").."\n"))
	end

	world.addEventHandler(eventHandler)

	-- for k, v in pairs(world.event) do env.info(string.format("[dcsd] %s = %s", k, v)) end
end
