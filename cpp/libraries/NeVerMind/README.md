This is  a very simple Non Volatile Memory library using flash.
Features :
- Storage entries are identified by an id between 1 and 65535
- Entry size between 1 byte and 64 bytes (can be changed)
- One sector at a time, garbage collecting to the next one (1024 bytes sector)
- Append only writing, the # of erase is proportional to sizeUsed/sectorSize

It is a bit robust but not that much, dont use it if you really really need 
the data stored.

