# # Bindgen of DLPack

import subprocess
import os
import shutil

path_cwd = os.path.abspath(os.getcwd())

# ## Bindgen configuration

# Users may change the following fields for their needs.

# Path for storing useful header files
path_header = f"{path_cwd}/../header"

# Path for temporary files
path_temp = f"{path_cwd}/tmp"

# Path for bindgen crate root
path_out = f"{path_cwd}/.."

# ## Copy necessary headers

# ### Copy to temporary directory

shutil.copytree(path_header, path_temp, dirs_exist_ok=True)

# +
# From now on, we will always work in temporary directory

os.chdir(path_temp)
# -

# ## Pre-Processing

with open("dlpack.h", "r") as f:
    token = f.read()

# ## Bindgen

subprocess.run([
    "bindgen",
    "dlpack.h", "-o", "dlpack.rs",
    "--allowlist-file", "dlpack.h",
    "--default-enum-style", "rust",
    "--no-layout-tests",
    "--use-core",
    "--merge-extern-blocks",
])

# ## Post-Processing

with open("dlpack.rs") as f:
    token = f.read()

token = token.replace("::core::ffi::", "").replace("::core::option::", "")
token = """
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
use core::ffi::*;

""" + token

with open("dlpack_ffi.rs", "w") as f:
    f.write(token)

shutil.copy(f"{path_temp}/dlpack_ffi.rs", f"{path_out}/src/lib.rs")

# ## Finalize

subprocess.run(["cargo", "fmt", "-p", "rstsr-dlpack-ffi"])
