MAX_REGISTER = 2 ^ 4
MAX_IMMEDIATE = 255
MIN_IMMEDIATE = -128
MAX_OFFSET = 7
MIN_OFFSET = -8
MAX_ADDRESS = 2 ^ 10

function populate_symbols(symbols, offset)
    offset = offset or 0
    local result = {}
    for index, symbol in ipairs(symbols) do
        result[symbol] = index - 1 + offset -- Lua indices start from 1, so subtract 1
    end
    return result
end

function populate_symbol_table()
    local symbols = {}

    local opcodes = {
        "nop", "hlt", "add", "sub", "nor", "and", "xor", "rsh",
        "ldi", "adi", "jmp", "brh", "cal", "ret", "lod", "str",
    }

    table.insert(symbols, populate_symbols(opcodes))

    local registers = {}
    for i = 1, 16 do
        table.insert(registers, 'r' .. tostring(i))
    end

    table.insert(symbols, populate_symbols(registers))

    local conditions = {
        { "eq",   "ne",      "ge",    "lt" },
        { "=",    "!=",      ">=",    "<" },
        { "z",    "nz",      "c",     "nc" },
        { "zero", "notzero", "carry", "notcarry" },
    }

    for _, con in ipairs(conditions) do
        table.insert(symbols, populate_symbols(con))
    end
end
