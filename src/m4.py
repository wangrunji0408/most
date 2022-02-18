import requests
import re
import time

M1 = 20220217214410
M2 = 104648257118348370704723119
M3 = 125000000000000140750000000000052207500000000006359661


def test237(n: int):
    c2, c3, c7 = 0, 0, 0
    while n % 3 == 0:
        n //= 3
        c2 += 1
    while n % 7 == 0:
        n //= 7
        c3 += 1
    while n % 11 == 0:
        n //= 11
        c7 += 1
    return c2, c3, c7


def test(n: int):
    if n % M1 == 0:
        return M1
    if n % M2 == 0:
        return M2
    if n % M3 == 0:
        return M3
    return test237(n)


nset = set()

while True:
    content = requests.get("http://47.95.111.217:10000/board.txt").text
    for s in re.findall(r'\n[0-9]+\n', content):
        n = int(s[1:-1])
        if n in nset:
            continue
        nset.add(n)
        res = test(n)
        print(n, res, flush=True)
    time.sleep(5)
