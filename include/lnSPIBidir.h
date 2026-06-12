/*
 *  (C) 2021/2022 MEAN00 fixounet@free.fr
 *  See license file
 *
 * Bidirectional SPI (full-duplex transceive + receive-only)
 *
 */

#pragma once
#include "lnSPI.h"

/**
 * @brief Extended SPI interface adding full-duplex transceive and receive-only operations
 *
 * This extends lnSPI with:
 * - transfer(): blocking full-duplex (TX + RX simultaneously)
 * - read():     blocking receive-only (sends 0xFF dummy bytes)
 * - asyncTransfer(): async full-duplex
 * - asyncRead():     async receive-only
 */
class lnSPIBidir : public lnSPI
{
  public:
    /**
     * @brief Blocking full-duplex transfer
     * @param nbBytes Number of bytes to exchange
     * @param dataOut Data to send (TX)
     * @param dataIn  Buffer for received data (RX)
     * @return true on success
     */
    virtual bool transfer(uint32_t nbBytes, const uint8_t *dataOut, uint8_t *dataIn) = 0;

    /**
     * @brief Blocking receive-only (sends 0xFF dummy bytes on MOSI)
     * @param nbBytes Number of bytes to receive
     * @param dataIn  Buffer for received data
     * @return true on success
     */
    virtual bool read(uint32_t nbBytes, uint8_t *dataIn) = 0;

    /**
     * @brief Async full-duplex transfer
     * @param nbBytes Number of bytes to exchange
     * @param dataOut Data to send (TX)
     * @param dataIn  Buffer for received data (RX)
     * @param cb      Callback on completion
     * @param cookie  User cookie for callback
     * @return true if started successfully
     */
    virtual bool asyncTransfer(uint32_t nbBytes, const uint8_t *dataOut, uint8_t *dataIn,
                               lnSpiCallback *cb, void *cookie) = 0;

    /**
     * @brief Async receive-only (sends 0xFF dummy bytes on MOSI)
     * @param nbBytes Number of bytes to receive
     * @param dataIn  Buffer for received data
     * @param cb      Callback on completion
     * @param cookie  User cookie for callback
     * @return true if started successfully
     */
    virtual bool asyncRead(uint32_t nbBytes, uint8_t *dataIn,
                           lnSpiCallback *cb, void *cookie) = 0;

    /**
     * @brief Clean up both TX and RX DMAs after async bidir operation
     * @return true on success
     */
    virtual bool finishAsyncDmaBidir() = 0;

    /**
     * @brief Factory: create a bidirectional SPI instance
     * @param instance SPI instance number (0, 1, ...)
     * @param pinCs    CS pin (-1 if none)
     * @return new lnSPIBidir instance, or NULL if not supported
     */
    static lnSPIBidir *createBiDir(uint32_t instance, int pinCs = -1);

  protected:
    lnSPIBidir(uint32_t instance, int pinCs = -1) : lnSPI(instance, pinCs) {}
};