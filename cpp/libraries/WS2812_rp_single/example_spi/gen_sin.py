import math

print("const uint8_t sin[181]={")
for i in range(0,181): # from 0 to 180 (included)
    ang = i*math.pi/180.
    s=(math.cos(ang)*128.)+0.5
    s=255-int(s)-127
    print(str(s),end="")
    print(",",end="")
    if (i+1)%16==15:
        print("")
print("};")
