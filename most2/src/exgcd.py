def exgcd(a, b):
    if b == 0:
        return 1, 0
    y, x = exgcd(b, a % b)
    y = y - (a // b * x)
    return x, y


def rev(a, b):
    x, _ = exgcd(a, b)
    return (x + b) % b


m1 = 7 * 887 * 24097
m2 = 104648257118348370704723401
m3_1 = 500000000000000221
m3_2 = 500000000000000231
m3_3 = 500000000000000243

print(rev(10, m1))
print(rev(10, m2))
print(rev(10, m3_1))
print(rev(10, m3_2))
print(rev(10, m3_3))
