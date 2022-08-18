import os

def clean(path,depth):
    print("curent path %s"%path)
    if(depth>3):
        return
    dir_file = os.listdir(path)
    if("Cargo.toml" in dir_file):
        os.system("cd %s && cargo clean"%path)
    if("Makefile" in dir_file):
        os.system("cd %s && make clean"%path)
    for fd in dir_file:
        if(not os.path.isdir(os.path.join(path, fd))):
            continue
        else:
            sub_path = os.path.join(path, fd)
            clean(sub_path,depth+1)


clean("/root/sgx/exp/other",0)