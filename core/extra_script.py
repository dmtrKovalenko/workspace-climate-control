import os
Import("env")

# include toolchain paths
env.Replace(CO2_COMPILATIONDB_INCLUDE_TOOLCHAIN=False)

# override compilation DB path
env.Replace(CO2_COMPILATIONDB_PATH=os.path.join("compile_commands.json"))
print("BUILD_DIR: ", env.get("COMPILATIONDB_PATH"))