import hashlib
from .consts import BECH32_CHARSET, BECH32_CONST, BECH32M_CONST, Encoding

def hash160(data):
    sha = hashlib.sha256(data).digest()
    return hashlib.new('ripemd160', sha).digest()

# --- Bech32/Bech32m Implementation ---

def bech32_polymod(values):
    GEN = [0x3b6a57b2, 0x26508e6d, 0x1ea119fa, 0x3d4233dd, 0x2a1462b3]
    chk = 1
    for v in values:
        b = chk >> 25
        chk = (chk & 0x1ffffff) << 5 ^ v
        for i in range(5):
            chk ^= GEN[i] if ((b >> i) & 1) else 0
    return chk

def bech32_hrp_expand(hrp):
    return [ord(x) >> 5 for x in hrp] + [0] + [ord(x) & 31 for x in hrp]

def bech32_create_checksum(hrp, data, spec):
    values = bech32_hrp_expand(hrp) + data
    const = BECH32M_CONST if spec == Encoding.BECH32M else BECH32_CONST
    polymod = bech32_polymod(values + [0, 0, 0, 0, 0, 0]) ^ const
    return [(polymod >> 5 * (5 - i)) & 31 for i in range(6)]

def local_bech32_encode(hrp, data, spec):
    combined = data + bech32_create_checksum(hrp, data, spec)
    return hrp + '1' + ''.join([BECH32_CHARSET[d] for d in combined])
