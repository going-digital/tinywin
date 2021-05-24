rem ..\tools\shader_minifier -o - --preserve-externals "src\shader.vtx" > src\shader_min.vtx
rem ..\tools\shader_minifier -o - --preserve-externals "src\shader.frag" > src\shader_min.frag

xargo rustc --release --target i686-pc-windows-msvc -- --emit=obj
xargo rustc --release --target i686-pc-windows-msvc -- --emit=asm
..\tools\crinkler /OUT:test.exe /SUBSYSTEM:WINDOWS target\i686-pc-windows-msvc\release\deps\miniwin.o  /ENTRY:mainCRTStartup "/LIBPATH:C:\Program Files (x86)\Windows Kits\10\Lib\10.0.17763.0\um\x86" gdi32.lib user32.lib opengl32.lib kernel32.lib

