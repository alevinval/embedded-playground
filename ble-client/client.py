import asyncio
from datetime import datetime
from bleak import BleakScanner, BleakClient

HUMIDITY = "987312e0-2354-11eb-9f10-fbc30a62cf38"

def callback(_sender, data):
    json_str = data.decode('utf-8');
    print(datetime.now(), " => ", json_str)

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
                results = await client.read_gatt_char(HUMIDITY)
                print(datetime.now(), "=>", results.decode('utf-8'))
                    # await client.start_notify(HUMIDITY, callback)
                # await asyncio.sleep(3)
        except:
            continue
            # print("disconnected", e)


asyncio.run(main())
