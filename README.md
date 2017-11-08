1. move this file into `%USERPROFILE%\Saved Games\DCS.openalpha\Scripts\` (or create a symbolic link, like: `New-Item -Path "M:\Saved Games\DCS.openalpha\Scripts\dcsd.lua" -ItemType HardLink -Value "M:\Desktop\dcsd\dcsd.lua"`)
2. go to DCS installation directory and add the following to `Scripts\MissionScripting.lua` (after `dofile('Scripts/ScriptingSystem.lua')`):

```lua
dofile(lfs.writedir().."Scripts/dcsd.lua")
```

3. add the following to the mission (in a `ONCE -> TIME MORE (1) -> DO SCRIPT` trigger):


```lua
dcsd.start(_G)
```
