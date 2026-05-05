#!/usr/bin/python3
import subprocess
import os


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
    subprocess.check_call(['mkdir',build_dir])
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
def build_all(category, title, use):

    top_working_dir=top_folder+"/"+category
    dbg("For "+category +" => working dir : "+str(top_working_dir))
    subprojects = grab_demo_folders(top_working_dir)

    for i in subprojects:
        shortname = i
        print("[",title, "/",use,"]:",shortname)
        os.chdir(top_working_dir)
        working_dir=top_working_dir+"/"+i
        build_name=title
        if False==build_single(working_dir, build_name, use):
            failures.append( str(title)+'_'+str(i)+str(use)) 
        #quit()
    pass
def build_all_bp(category ):
    build_all(category, "ARM_M4",["GD32F3"])
    build_all(category, "ARM_M4",["CLANG","GD32F3"])
    build_all(category, "ARM_M3",["CLANG"])
    build_all(category, "ARM_M3",[])
    build_all(category, "RISCV_CH32V3x",["CLANG","CH32V3x"])
#
#
#________________________________________
top_folder=os.path.abspath(os.getcwd()+'/../demoProject/')
top_esprit=os.path.abspath(os.getcwd()+'/../../esprit')

dbg("Esprit:"+top_esprit)
dbg( "Top Folder : "+top_folder)
print("-- Build all --")
if True:
    build_all_bp("common")
# RP
if True:
    build_all("rp", "ARM_M33",["RP2350","CLANG"])
    build_all("rp", "ARM_M0",["RP2040","CLANG"])

if True:
    build_all_bp("bp")


print("-- Failures --")
for i in failures:
    print("\t"+i)
print("-- Done --")

