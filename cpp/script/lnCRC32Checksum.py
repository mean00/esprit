#/usr/bin/python3
import sys
import crcmod

header_size=4
total_header_size=header_size+2*4
#
#_____________
def usage():
    print("lnCH32Checksum-crc32 : Add a checksum to the fw to check its integrity");
    print("\tlnCH32Checksum-crc32.py intput_bin_file checksumed_bin_file\n");
#
#_____________
def write32(f,v):
    n_byte = v.to_bytes(4,byteorder='little')
    f.write(n_byte)
#
#___________________
if(len(sys.argv)!=3):
    usage();
    exit(-1);

infile=sys.argv[1];
outfile=sys.argv[2];

print("lnCH32Checksum-crc32:  checksuming %s => %s " %(infile,outfile))


print("Reading file..")
file=open(infile,"rb")
content=bytearray(file.read(-1))
file.close();

binSize=len(content)

print("%d bytes read " % (binSize))

binSize-=total_header_size # Skip 4 first words = MSP / Reset / size / xxhash

payload = content[total_header_size:]
hashed = content[total_header_size:]
for i in range(binSize>>2):
    a0=hashed[i*4+0]
    a1=hashed[i*4+1]
    a2=hashed[i*4+2]
    a3=hashed[i*4+3]
    hashed[i*4+0]=a3
    hashed[i*4+1]=a2
    hashed[i*4+2]=a1
    hashed[i*4+3]=a0
# Doing it in big endian would be faster , did not find out how
crc32 = crcmod.predefined.mkCrcFun('crc-32-mpeg')
lnHash=crc32(hashed)
print("zlib mpeg2 reverted digest %x " % (lnHash))

result=open(outfile,"wb")
result.write(content[0:4])
write32(result,binSize)
write32(result,lnHash)
result.write(payload)
result.close()
print("Done")


