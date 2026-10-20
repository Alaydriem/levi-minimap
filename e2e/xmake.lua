add_rules("mode.debug", "mode.release")

set_runtimes("MD")

target("lm-e2e")
    set_kind("binary")
    set_languages("c++20")
    set_toolchains("clang-cl")
    add_defines("NOMINMAX", "UNICODE")
    add_cxflags("/utf-8", "/EHsc")
    add_files("src/*.cpp")
    add_includedirs("src", "../shim/include")
    add_linkdirs("$(projectdir)/../target/release")
    add_links("minimap_ffi")
    -- relink whenever cargo rebuilds the Rust library
    on_load(function(target)
        target:data_add("linkdepfiles", path.join(os.projectdir(), "..", "target", "release", "minimap_ffi.lib"))
    end)
    add_syslinks("kernel32", "ntdll", "userenv", "ws2_32", "dbghelp", "advapi32", "bcrypt")
