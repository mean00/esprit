

#pragma once
#include "lnAssert.h"
#include "stddef.h"
#include "stdint.h"
/**
 * @brief
 *
 */
class lnRingBuffer
{
  public:
    /**
     * @brief Construct a new ln Ring Buffer object
     *
     * @param size
     */
    lnRingBuffer(uint32_t size)
    {
        uint32_t c = size;
        xAssert(size == (size & ~(size - 1))); // make sure it is a power of 2
        _size = size;
        _mask = size - 1;
        _head = _tail = 0;
        _buffer = new uint8_t[size];
    }
    /**
     * @brief Destroy the ln Ring Buffer object
     *
     */
    virtual ~lnRingBuffer()
    {
        delete[] _buffer;
        _buffer = NULL;
    }
    /**
     * @brief
     *
     * @return int
     */
    uint32_t size() const
    {
        return _size;
    }
    /**
     * @brief
     *
     * @return true
     * @return false
     */
    bool empty() const
    {
        return (_head == _tail);
    }
    /**
     * @brief
     *
     * @return true
     * @return false
     */
    bool full() const
    {
        return _head == ((_tail + _size - 1) & _mask);
    }
    /**
     * @brief
     *
     * @return int
     */
    uint32_t count() const
    {
        return (_size + _head - _tail) & _mask;
    }
    /**
     * @brief
     *
     * @return int
     */
    uint32_t free() const
    {
        return _size - count() - 1;
    }
    /**
     * @brief
     *
     * @param a
     * @param b
     * @return int
     */
    static inline uint32_t CMIN(uint32_t a, uint32_t b)
    {
        if (a <= b)
            return a;
        return b;
    }
    /**
     * @brief
     *
     */
    void flush()
    {
        _tail = _head = 0;
    }
    /**
     * @brief
     *
     * @return uint8_t*
     */
    uint8_t *ringBuffer() const
    {
        return _buffer;
    }

    //----------------------------------
    void consume(uint32_t n)
    {
        _tail += n;
        _tail &= _mask;
    }
    /**
     * @brief Get the Read Pointer object
     *
     * @param to
     * @return int
     */
    uint32_t getReadPointer(uint8_t **to)
    {
        *to = _buffer + _tail;
        if (_head < _tail)
            return _size - _tail;
        // if (_head >= _tail)
        return _head - _tail;
    }
    uint32_t getWritePointer(uint8_t **to) const
    {
        *to = _buffer + _head;
        if (_head < _tail)
            return _tail - _head - 1;
        // if (_head >= _tail)
        uint32_t res = _size - _head;
        if (_tail == 0)
            res--;
        return res;
    }
    /**
     * @brief
     *
     * @param insize
     * @param data
     * @return true
     * @return false
     */
    uint32_t put(uint32_t insize, const uint8_t *data)
    {
        uint8_t *to;
        uint32_t done = 0;
        uint32_t orgsize = insize;
        uint32_t n = getWritePointer(&to);
        if (!n)
            return done;
        if (n > insize)
            n = insize;
        memcpy(to, data, n);
        insize -= n;
        data += n;
        done += n;
        add(n);

        if (!insize)
            return done;

        // in case we wrapped...
        n = getWritePointer(&to);
        if (!n)
            return done;
        if (n > insize)
            n = insize;
        memcpy(to, data, n);
        insize -= n;
        data += n;
        done += n;
        add(n);

        return done;
    }
    /**
     * @brief
     *
     * @param size
     * @param data
     * @return int
     */
    uint32_t get(uint32_t size, uint8_t *data)
    {
        size = CMIN(size, count());
        uint32_t orgsize = size;
        uint8_t *from;
        uint32_t done = 0;
        uint32_t n = getReadPointer(&from);
        if (!n)
            return done;
        if (n > size)
            n = size;
        memcpy(data, from, n);
        size -= n;
        data += n;
        done += n;
        consume(n);

        if (!size)
            return done;

        n = getReadPointer(&from);
        if (!n)
            return done;
        if (n > size)
            n = size;
        memcpy(data, from, n);
        size -= n;
        data += n;
        done += n;
        consume(n);
        return done;
    }
    //----------------------------------
  protected:
    void add(uint32_t n)
    {
        _head += n;
        _head &= _mask;
    }

  protected:
    volatile uint32_t _head, _tail;
    uint32_t _mask, _size;
    uint8_t *_buffer;
    //
};
