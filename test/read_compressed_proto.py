import example_pb2
from read_compressed_data import read_file

for data in read_file("../tcp___localhost_5556_test.rec"):
    decompressed_data = gzip.decompress(data)
    loaded_address_book = example_pb2.AddressBook()
    loaded_address_book.ParseFromString(decompressed_data)
    print(loaded_address_book)