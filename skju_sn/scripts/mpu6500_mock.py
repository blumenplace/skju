if request.isInit and not request.isRead and not request.isWrite:
    pass
elif request.Connection != null and isinstance(request.Connection, SPITransaction):
    transaction = request.Connection
    if transaction.IsCommand:
        self.Log(LogLevel.Debug, "SPI command: {0}", transaction.ReadBytes(0, transaction.CommandLength))
    elif transaction.IsData:
        data = transaction.ReadBytes(transaction.CommandLength, transaction.DataLength)
        self.Log(LogLevel.Debug, "SPI data received: {0}", data)
        # Echo back data
        transaction.WriteBytes(transaction.CommandLength, data)
