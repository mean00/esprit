#!/usr/bin/python3
import subprocess
import os
import argparse
import sys

blacklist = ['cmake','everything','usb'] #,'hello']
failures = []
#
#
#_____________________________________

def dbg(c):
    #print(str(c))
    pass
#
#
#_____________________________________

def grab_demo_folders(path):
    projects = []
    dbg("Scanning "+path)
    for item in os.scandir(path):
        if item.is_dir():
            if not (item.name in blacklist):
                #print(item.name)
                projects.append(item.name)
    projects.sort()
    dbg(str(projects))
    return projects
#
#
#_____________________________________

def exec_makethingie(title, cmds):
    res=0
    print("\t"+title+"...",end='')
    dbg(str(cmds))
    dbg(os.getcwd())
    out=""
    try:
        res=0 # Assume it raises an excception if a problem occurs
        #subprocess.check_call(cmds,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL) #, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        out=subprocess.check_output(cmds,stderr=subprocess.STDOUT)
    except Exception as e:
        print("\t OS ERROR" +str(e))
        dbg("\t OS ERROR" +str(out))
        res=-1
    if res!=0:
        print("\t"+title+"FAILED")
        return False
    print("\t\tok")
    return True
#
#
#_____________________________________

def build_single(  working_dir, build_name, extra_args):
    os.chdir(working_dir)
    dbg(build_name+":"+os.getcwd())
    #-- Cleanup
    build_dir=working_dir+'/build'+'_'+str(build_name)
    esprit_dir=working_dir+'/esprit'
    for i in extra_args:
        build_dir=build_dir+'_'+str(i) 
    subprocess.check_call(['rm','-Rf',build_dir])
    subprocess.check_call(['rm','-f','esprit'])
    subprocess.check_call(['ln','-s',top_esprit,'esprit'])
    os.makedirs(build_dir, exist_ok=True)
    os.chdir(build_dir)
    dbg(os.getcwd())
    #
    cmakeParam= [ 'cmake', '-G', 'Ninja']
    for i in extra_args:
        cmakeParam.append('-DUSE_'+i+'=True')
        cmakeParam.append('-DCMAKE_CXX_COMPILER_LAUNCHER=ccache -D CMAKE_C_COMPILER_LAUNCHER=ccache')
    cmakeParam.append('..')

    makeParam = ['ninja','-v']

    dbg(cmakeParam)
   # return False
    
    if exec_makethingie("CMAKE",cmakeParam)==False:
        return False
    if exec_makethingie("BUILD",makeParam)==False:
        return False
    #if it i sok, cleanup
   
    subprocess.check_call(['rm','-Rf',build_dir])
    subprocess.check_call(['unlink',esprit_dir])
    return True
    #os.chdir(cwd)
    #print(str(projects))
#
#
#_____________________________________
def build_all(category, title, extra_args):

    top_working_dir=top_folder+"/"+category
    dbg("For "+category +" => working dir : "+str(top_working_dir))
    subprojects = grab_demo_folders(top_working_dir)

    for i in subprojects:
        shortname = i
        print("[",title, "/",extra_args,"]:",shortname)
        os.chdir(top_working_dir)
        working_dir=top_working_dir+"/"+i
        build_name=title
        if False==build_single(working_dir, build_name, extra_args):
            failures.append((category, i, title, extra_args))
        #quit()
    pass

def get_build_configs(cpu_families, compilers):
    """Generate the build configurations (category, build_name, extra_args) based on options."""
    configs = []
    gcc = (compilers == 'both' or compilers == 'gcc')
    clang = (compilers == 'both' or compilers == 'clang')

    # For common & bp categories
    for cat in ('common', 'bp'):
        # stm32 (generic ARM)
        if 'stm32' in cpu_families:
            if gcc:
                configs.append((cat, "ARM_M3", []))
            if clang:
                configs.append((cat, "ARM_M3", ["CLANG"]))
        # GD32
        if 'gd32' in cpu_families:
            if gcc:
                configs.append((cat, "ARM_M4", ["GD32F3"]))
            if clang:
                configs.append((cat, "ARM_M4", ["CLANG", "GD32F3"]))
        # CH32
        if 'ch32' in cpu_families:
            if clang:   # CH32 currently only built with Clang in the script
                configs.append((cat, "RISCV_CH32V3x", ["CLANG", "CH32V3x"]))

    # RP series (only for 'rp' category)
    if 'rp2040' in cpu_families:
        if clang:   # RP2040 only built with Clang here
            configs.append(("rp", "ARM_M0", ["CLANG", "RP2040"]))
    if 'rp2350' in cpu_families:
        if clang:   # RP2350 only built with Clang here
            configs.append(("rp", "ARM_M33", ["CLANG", "RP2350"]))

    return configs

def parse_args():
    parser = argparse.ArgumentParser(
        description="Build Swindle demo projects with selected compiler and CPU")
    parser.add_argument('--compiler', choices=['gcc', 'clang', 'both'], default='both',
                        help="Compiler to use (default: both)")
    parser.add_argument('--cpu', nargs='+', default=['all'],
                        choices=['stm32', 'gd32', 'ch32', 'rp2040', 'rp2350', 'all'],
                        help="CPU families to build (default: all)")
    parser.add_argument('--list', action='store_true',
                        help="Only list what would be built")
    return parser.parse_args()

#
#
#________________________________________
if __name__ == "__main__":
    args = parse_args()

    top_folder=os.path.abspath(os.getcwd()+'/../demoProject/')
    top_esprit=os.path.abspath(os.getcwd()+'/../../esprit')

    dbg("Esprit:"+top_esprit)
    dbg( "Top Folder : "+top_folder)

    cpu_families = args.cpu
    if 'all' in cpu_families:
        cpu_families = ['stm32', 'gd32', 'ch32', 'rp2040', 'rp2350']

    configs = get_build_configs(cpu_families, args.compiler)

    if args.list:
        print("Build plan:")
        for cat, name, extras in configs:
            print(f"  Category: {cat}, Build: {name}, Flags: {extras}")
        sys.exit(0)

    print("-- Build all --")
    for cat, name, extras in configs:
        build_all(cat, name, extras)

    # Summary
    print("\n-- SUMMARY --")
    if failures:
        print("Failures (" + str(len(failures)) + "):")
        for cat, proj, name, extras in failures:
            extra_str = ', '.join(extras) if extras else "none"
            print(f"  * {cat}/{proj} [{name}] with flags: {extra_str}")
    else:
        print("All builds succeeded.")
    print("-- Done --")
