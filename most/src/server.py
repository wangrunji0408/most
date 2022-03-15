import asyncio
import random
import string
import re

N = 256
M1 = 20220217214410
M2 = 104648257118348370704723119
M3 = 125000000000000140750000000000052207500000000006359661
M4 = (3 ** 50) * (7 ** 30) * (11 ** 20)

RATE = 0.05

def gen():
    p = random.random()
    if p < RATE:
        return str(M1 * (1 << random.randrange(1, 1000)))
    elif p < RATE * 2:
        return str(M2 * (1 << random.randrange(1, 1000)))
    elif p < RATE * 3:
        return str(M3 * (1 << random.randrange(1, 1000)))
    elif p < RATE * 4:
        return str(M4 * (1 << random.randrange(1, 1000)))
    else:
        return ''.join(random.choice(string.digits) for i in range(random.randrange(100, 300)))


async def send_data(reader, writer):
    writer.write(b'HTTP/1.1 200 OK\r\nServer: Most\r\nContent-type: text/plain\r\n\r\n')
    await writer.drain()

    s = ""
    while True:
        l = random.randrange(50, 300)
        while len(s) < l:
            s += gen()
        bs = s[:l]
        s = s[l:]
        writer.write(bs.encode('utf-8'))
        try:
            await writer.drain()
        except:
            return
        await asyncio.sleep(random.random() * 0.3 + 0.2)


async def recv_ans(reader, writer):
    while True:
        header = await reader.readuntil(b'\r\n\r\n')
        matches = re.findall(r'Content-Length: ([0-9]+)', str(header))
        l = int(matches[0])
        data = await reader.read(l)
        print(data)


async def main():
    server = await asyncio.start_server(send_data, '127.0.0.1', 10001)
    server1 = await asyncio.start_server(recv_ans, '127.0.0.1', 10002)
    asyncio.create_task(server1.serve_forever())

    addrs = ', '.join(str(sock.getsockname()) for sock in server.sockets)
    print(f'Input serving on {server.sockets[0].getsockname()}')
    print(f'Output serving on {server1.sockets[0].getsockname()}')

    async with server:
        await server.serve_forever()

asyncio.run(main())
