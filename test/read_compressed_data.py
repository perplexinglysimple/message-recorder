import struct
import gzip


def read_file(filename: str):
    with open(filename, "rb") as f:
        data = f.read(8)
        if len(data) != 8:
            return
        size = struct.unpack("!Q", data)[0]
        while True:
            data = f.read(size)
            if len(data) != size:
                return
            yield data
            data = f.read(8)
            if len(data) != 8:
                return
            size = struct.unpack("!Q", data)[0]


for data in read_file("../tcp___localhost_5557_test.rec"):
    print(gzip.decompress(data).decode())
