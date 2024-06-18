import asyncio
from datetime import datetime
from bleak import BleakScanner, BleakClient

HUMIDITY_CHAR = "987312e0-2354-11eb-9f10-fbc30a62cf38"


def save_data(data: bytearray):
    data: str = data.decode('utf-8')
    with open('data.csv', 'a') as f:
        content = f"{datetime.now()},{data}\n"
        print("writing contents", content)
        f.write(content)


async def find_device_by_name(name):
    devices = await BleakScanner.discover()
    for device in devices:
        if device.name == name:
            return device
    return None


async def main():
    while True:
        try:
            device = await find_device_by_name("esp32s3")
            if not device:
                continue

            print(f"connecting to {device.name}")
            async with BleakClient(device.address) as client:
                raw_bytes: bytearray = await client.read_gatt_char(HUMIDITY_CHAR)
                save_data(raw_bytes)
        except Exception as _:
            continue

asyncio.run(main())
