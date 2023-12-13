import os
Import("env")

# include toolchain paths
env.Replace(COMPILATIONDB_INCLUDE_TOOLCHAIN=False)

# override compilation DB path
env.Replace(COMPILATIONDB_PATH=os.path.join("compile_commands.json"))
print("BUILD_DIR: ", env.get("COMPILATIONDB_PATH"))
