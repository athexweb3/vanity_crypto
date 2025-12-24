
try:
    from tonsdk.contract.wallet import Wallets, WalletVersionEnum
    from tonsdk.boc import Cell
except ImportError:
    print("Error: 'tonsdk' imports failed.")
    exit(1)

def generate_ton_address():
    print("Generating TON Wallet V4R2...")
    
    # helper to create random wallet
    # This automatically generates mnemonic, keys, and state_init
    mnemonics, pub_k, priv_k, wallet = Wallets.create(WalletVersionEnum.v4r2, workchain=0)
    
    address = wallet.address.to_string(is_user_friendly=True, is_url_safe=True, is_bounceable=False)
    
    print(f"Private Key (Hex): {priv_k.hex()}")
    print(f"Private Key Len:   {len(priv_k)} bytes")
    print(f"Public Key (Hex):  {pub_k.hex()}")
    print(f"Public Key Len:    {len(pub_k)} bytes")
    print(f"Address:           {address}")
    
    # helper
    def ensure_cell(obj):
        if isinstance(obj, Cell): return obj
        # if string/bytes looking like BOC
        if isinstance(obj, str):
            obj = bytes.fromhex(obj)
        return Cell.one_from_boc(obj)

    # Inspect Hashes
    state_init_dict = wallet.create_state_init()
    code_cell = ensure_cell(state_init_dict['code'])
    data_cell = ensure_cell(state_init_dict['data'])
    state_init_cell = ensure_cell(state_init_dict['state_init'])
    
    print(f"Cell methods: {dir(code_cell)}")
    # Try finding hash method
    if hasattr(code_cell, 'bytes_hash'):
        code_hash = code_cell.bytes_hash()
    else:
        # Fallback inspection
        code_hash = b'\x00' * 32
    
    # code_hash = code_cell.hash()
    # data_hash = data_cell.hash()
    # state_init_hash = state_init_cell.hash()
    
    code_hash = code_cell.bytes_hash()
    data_hash = data_cell.bytes_hash()
    state_init_hash = state_init_cell.bytes_hash()

    print(f"Code Hash (Hex):       {code_hash.hex()}")
    print(f"Data Hash (Hex):       {data_hash.hex()}")
    print(f"StateInit Hash (Hex):  {state_init_hash.hex()}")

    # DUMP TEMPLATES
    print(f"Data Cell Bytes (Hex):      {data_cell.bytes_repr().hex()}")
    print(f"StateInit Cell Bytes (Hex): {state_init_cell.bytes_repr().hex()}")

if __name__ == "__main__":
    generate_ton_address()
