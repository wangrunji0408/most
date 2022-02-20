import requests

N = 256
M = 20220217214410
user = "user"
passwd = "passwd"

s = b""
with requests.Session().get("http://172.1.1.119:10001", stream=True, headers=None) as fin:
    for c in fin.iter_content():
        s += c
        if len(s) > N:
            s = s[-N:]
        for i in range(len(s)):
            if s[i] != ord("0") and int(s[i:]) % M == 0:
                requests.post(f"http://172.1.1.119:10002/submit?user={user}&passwd={passwd}", data=s[i:])
                print("submit", s[i:].decode("ascii"))
