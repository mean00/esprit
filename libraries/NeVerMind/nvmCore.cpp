/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */
#include "nvmCore.h"
#include "esprit.h"
#include "nvmCore_internal.h"

#if 0
#define VERBOSE Logger
#else
#define VERBOSE(...)                                                                                                   \
    {                                                                                                                  \
    }
#endif

#if 1
#define INFO Logger
#else
#define INFO(...)                                                                                                      \
    {                                                                                                                  \
    }
#endif

/**
 *
 * @param nbSectors
 */
lnNvm::lnNvm(uint32_t nbSectors)
{
    _nbSectors = nbSectors;
    _currentSector = NVM_INVALID_SECTOR;
    _readIndex = 0;
    _writeIndex = 0;
}

#define LN_NVM_ALLONES 0xffff

/**
 *
 */
lnNvm::~lnNvm()
{
}

/**
 *
 * @return
 */
bool lnNvm::begin()
{
    // find first active sector
    int sec = -1;
    for (int i = 0; i < _nbSectors && sec == -1; i++)
    {
        if (getSectorState(i) != Nvm_Active)
            continue;
        // Does it look correct ?
        _currentSector = i;
        if (sanityCheck())
            sec = i;
        else // erase it to remove the zeroes
        {
            eraseSector(i);
        }
    }
    //
    if (sec == -1)
    {
        _currentSector = NVM_INVALID_SECTOR;
        Logger("No valid sector found, reformating\n");
        return false;
    }
    return true;
}
/**
 */
bool lnNvm::sanityCheck()
{
    if (_currentSector == NVM_INVALID_SECTOR)
        return false;
    INFO("Starting sanity Check\n");
    int offset = 4;
    LN_NVM_ENTRY entry;
    bool run = true;
    int processed = 0;
    bool valid = false;
    while (run && offset < (LN_NVM_SECTOR_SIZE - 8 - 2))
    {
        if (!readSector(_currentSector, offset, 8, (uint8_t *)&entry))
        {
            run = false;
            continue;
        }
        int roundup = (entry.size + 1) & 0xfe;
        if (LN_NVM_ALLONES == entry.id) // last entry reached
        {
            valid = true;
            break;
        }
        // all zeroes are forbiden
        if (0 == entry.id && 0 == entry.size && 0 == entry.state && 0 == entry.validated)
        {
            // First one is inconsistent, drop here
            if (!processed)
                return false;
            break;
        }
        if (entry.size > LN_NVM_MAX_PAYLOAD)
        {
            VERBOSE("Payload exceeded %x :%d \n", offset, entry.size);
            run = false;
            continue;
        }

        offset = offset + 8 + roundup;
        processed++;
    }
    if (valid == true)
    {
        INFO("Check ok\n");
        _writeIndex = offset;
    }
    _readIndex = offset;
    INFO("read Offset =0x%x \n", _readIndex);
    return true;
}
/**
 * Prepare sector 0
 * @return
 */
bool lnNvm::format()
{
    lnAutoMutex aut(&_mutex);
    _currentSector = NVM_INVALID_SECTOR;
    // Prepare sector 0
    if (!prepareSector(0))
    {
        return false;
    }
    if (!setSectorState(0, Nvm_Active))
    {
        return false;
    }
    _currentSector = 0;
    // init read & write index
    _writeIndex = 4;
    _readIndex = 4;
    return true;
}
/**
 *
 * @param id
 * @param size
 * @param data
 * @return
 */

bool lnNvm::findEntry(int id, uint32_t &offset, LN_NVM_ENTRY &entry)
{
    if (_currentSector == NVM_INVALID_SECTOR)
        return false;

    bool run = true;
    int processed = 0;
    offset = 4;
    while (run && offset < (LN_NVM_SECTOR_SIZE - 8 - 2))
    {
        if (!readSector(_currentSector, offset, 8, (uint8_t *)&entry))
        {
            return false;
        }
        if (LN_NVM_ALLONES == entry.id)
            return false;
        int roundup = (entry.size + 1) & 0xfe;
        if (roundup > LN_NVM_MAX_PAYLOAD)
            return false;
        if (entry.state != 0 && entry.state != LN_NVM_ALLONES)
        {
            Logger("Broken entry at offset 0x%x\n", offset);
            return false;
        }
        if (id != entry.id || entry.state == 0 || entry.validated != 0)
        {
            offset += 8 + roundup;
            continue;
        }
        if (LN_NVM_ALLONES == entry.state) // valid
            return true;
        // invalid
        offset += 8 + roundup;
        continue;
    }
    Logger("Entry %d not found\n", id);
    return false;
}
/**
 *
 * @param id
 * @param size
 * @param data
 * @return
 */
bool lnNvm::read(uint32_t id, uint32_t size, uint8_t *data)
{
    if (!id)
    {
        Logger("NVM: ID is forbidden\n");
        return false;
    }
    if (_currentSector == NVM_INVALID_SECTOR)
        return false;
    LN_NVM_ENTRY entry;
    uint32_t offset;
    if (!findEntry(id, offset, entry))
        return false;

    if (entry.size != size)
    {
        Logger("Inconistent size\n");
        return false;
    }
    return readSector(_currentSector, offset + 8, size, data);
}
/**
 *
 * @param address
 * @return
 */
bool lnNvm::getWriteAddress(uint32_t &offset)
{
    if (_currentSector == NVM_INVALID_SECTOR)
        return false;

    LN_NVM_ENTRY entry;
    bool run = true;
    int processed = 0;
    offset = 4;
    while (run && offset <= (LN_NVM_SECTOR_SIZE - 8 - 2))
    {
        if (!readSector(_currentSector, offset, 8, (uint8_t *)&entry))
        {
            return false;
        }
        if (LN_NVM_ALLONES == entry.id && LN_NVM_ALLONES == entry.size)
            return true; // got it
        if (entry.size > LN_NVM_MAX_PAYLOAD)
            return false;
        int roundup = (entry.size + 1) & 0xfe;
        if (roundup > LN_NVM_MAX_PAYLOAD)
            return false;
        if (entry.state != 0 && entry.state != LN_NVM_ALLONES)
        {
            Logger("Broken entry at offset 0x%x\n", offset);
            return false;
        }
        // invalid
        offset += 8 + roundup;
        continue;
    }
    Logger("getWriteAddress:: failed\n");
    return false;
}
/**
 *
 * @param id
 * @param size
 * @param data
 * @return
 */
bool lnNvm::write(uint32_t id, uint32_t size, uint8_t *data)
{
    if (_currentSector == NVM_INVALID_SECTOR)
    {
        INFO("Write  : invalid sector\n");
        return false;
    }
    if (!id)
    {
        Logger("NVM: ID is forbidden\n");
        return false;
    }

    if (_writeIndex <= 0)
    {
        INFO("Cannot write, write index is invalid\n");
        return false;
    }
    uint32_t oldOffset;
    LN_NVM_ENTRY entry;

    //
    int roundup = (size + 1) & 0xfe;
    if ((_writeIndex + 8 + roundup) >= (LN_NVM_SECTOR_SIZE) * 9 / 10)
    {
        if (!garbageCollection())
        {
            INFO("Garbage collection failed \n");
            return false;
        }
        if (!sanityCheck())
        {
            INFO("sanityCheck  failed \n");
            return false;
        }
        if (_writeIndex <= 0)
        {
            INFO("sanityCheck (2)  failed \n");
            return false;
        }
    }
    bool hasOldEntry = findEntry(id, oldOffset, entry);
    entry.id = id;
    entry.size = size;
    entry.state = LN_NVM_ALLONES;
    entry.validated = LN_NVM_ALLONES;
    if (!writeSector(_currentSector, _writeIndex, 8, (uint8_t *)&entry))
    {
        _writeIndex = 0;
        return false;
    }
    if (!writeSector(_currentSector, _writeIndex + 8, roundup, data))
    {
        _writeIndex = 0;
        return false;
    }
    // fully written we can write validated too
    uint16_t vd = 0;
    if (!writeSector(_currentSector, _writeIndex + LN_NVM_VALIDATED_OFFSET, 2, (uint8_t *)&vd))
    {
        _writeIndex = 0;
        return false;
    }
    _writeIndex += 8 + roundup;
    if (hasOldEntry)
    {
        uint16_t v = 0;
        if (!writeSector(_currentSector, oldOffset + LN_NVM_STATE_OFFSET, 2, (uint8_t *)&v))
        {
            INFO("Erasing old value failed \n");
            return false; // invalidate old entry
        }
    }
    INFO("OK\n");
    return true;
}

/**
 *
 * @param sector
 * @return
 */
bool lnNvm::prepareSector(uint32_t sector)
{
    if (!eraseSector(sector))
        return false;
    return setSectorState(sector, Nvm_Prep);
}
/**
 *
 * @param sector
 * @param state
 * @return
 */
bool lnNvm::setSectorState(uint32_t sector, LN_NVM_SECTOR_STATE state)
{
    LN_NVM_SECTOR_STATE oldState = getSectorState(sector);
    uint16_t val = 0;
    if (oldState == state)
        return true;
    switch (state + oldState * 16)
    {
    case 1: // erase=>prep
        return writeSector(sector, 0, 2, (uint8_t *)&val);
        break;
    case 18: // pre => active
        return writeSector(sector, 2, 2, (uint8_t *)&val);
        break;
    default:
        return false;
        break;
    }
}
/**
 *
 * @param sector
 * @return
 */
LN_NVM_SECTOR_STATE lnNvm::getSectorState(uint32_t sector)
{
    uint16_t a, b;
    if (!readSector(sector, 0, 2, (uint8_t *)&a))
        return Nvm_Error;
    if (!readSector(sector, 2, 2, (uint8_t *)&b))
        return Nvm_Error;
    if (0 == a && LN_NVM_ALLONES == b)
        return Nvm_Prep;
    if (0 == a && 0 == b)
        return Nvm_Active;
    if (LN_NVM_ALLONES == a && LN_NVM_ALLONES == b)
        return Nvm_Erased;
    return Nvm_Error;
}
/**
 *
 * @return
 */
bool lnNvm::garbageCollection()
{
    if (_currentSector == NVM_INVALID_SECTOR)
        return false;

    uint8_t tmp[LN_NVM_MAX_PAYLOAD];
    int offset = 4;
    int otherSector = (_currentSector + 1) % _nbSectors;
    if (!prepareSector(otherSector))
        return false;

    LN_NVM_ENTRY entry;
    bool run = true;
    int processed = 0;
    int writeOffset = 4;
    VERBOSE("Garbage collecting\n");
    while (run && offset < (_writeIndex))
    {
        if (!readSector(_currentSector, offset, 8, (uint8_t *)&entry))
        {
            run = false;
            continue;
        }
        int roundup = (entry.size + 1) & 0xfe;
        if (LN_NVM_ALLONES == entry.id)
        {
            VERBOSE("End of scan at offset %x  \n", offset);
            break;
        }
        if (entry.size > LN_NVM_MAX_PAYLOAD)
        {
            VERBOSE("Payload exceeded %x :%d \n", offset, entry.size);
            run = false;
            continue;
        }
        if (LN_NVM_ALLONES != entry.state || 0 != entry.validated)
        {
            VERBOSE("Invalid entry at %x  :%x  \n", offset, entry.state);
            offset = offset + 8 + roundup;
            continue;
        }
        if (!readSector(_currentSector, offset + 8, entry.size, tmp))
        {
            VERBOSE("Read payload error entry at %x  :%x \n", offset);
            run = false;
            continue;
        }
        INFO("Found valid entry %d  \n", entry.id);
        offset = offset + 8 + roundup;
        // Write
        entry.state = LN_NVM_ALLONES;
        if (!writeSector(otherSector, writeOffset, 8, (uint8_t *)&entry))
        {
            run = false;
            continue;
        }

        INFO("Garbage collected id=%d  \n", entry.id);

        if (!writeSector(otherSector, writeOffset + 8, roundup, tmp))
        {
            run = false;
            continue;
        }

        // Validate it
        uint16_t vd = 0;
        if (!writeSector(otherSector, writeOffset + LN_NVM_VALIDATED_OFFSET, 2, (uint8_t *)&vd))
        {
            run = false;
            continue;
        }

        writeOffset += 8 + roundup;
        processed++;
    }
    Logger("Garbage collecting done %d entries processed\n", processed);
    INFO("Switching to sector %d  \n", otherSector);
    if (!setSectorState(otherSector, Nvm_Active))
        return false;
    int previousCurrent = _currentSector;

    INFO("Erasing old  sector %d  \n", previousCurrent);
    _currentSector = otherSector;
    eraseSector(previousCurrent);
    INFO("New current sector= %d   \n", _currentSector);
    Logger("Garbage collecting ok\n");
    return true;
}
/**
 */
bool lnNvm::dumpEntries(uint32_t sector)
{
    if (sector == -1)
        sector = _currentSector;
    bool run = true;
    int processed = 0;
    int offset = 4;
    LN_NVM_ENTRY entry;
    while (run && offset < (LN_NVM_SECTOR_SIZE - 8 - 2))
    {
        if (!readSector(sector, offset, 8, (uint8_t *)&entry))
        {
            return false;
        }
        if (LN_NVM_ALLONES == entry.id)
            return false;
        int roundup = (entry.size + 1) & 0xfe;
        if (roundup > LN_NVM_MAX_PAYLOAD)
            return false;
        if (entry.state != 0 && entry.state != LN_NVM_ALLONES)
        {
            Logger("Broken entry at offset 0x%x\n", offset);
            return false;
        }
        if (entry.state == LN_NVM_ALLONES)
        {
            Logger("Offset=%x  \n", offset);
            Logger("ID    =%x  \n", entry.id);
            Logger("size  =%x  \n", entry.size);
            Logger("state =%x  \n", entry.state);
        }
        // invalid
        offset += 8 + roundup;
        continue;
    }
    return true;
}
// EOF
