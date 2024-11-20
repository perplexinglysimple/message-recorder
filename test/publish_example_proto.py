import zmq
import time
import example_pb2


def random_name():
    first_names = ["Alice", "Bob", "Charlie", "Diana", "Edward"]
    last_names = ["Smith", "Johnson", "Williams", "Jones", "Brown"]
    return f"{random.choice(first_names)} {random.choice(last_names)}"


def random_email(name):
    domains = ["example.com", "test.com", "email.com"]
    return f"{name.replace(' ', '.').lower()}@{random.choice(domains)}"


def random_phone():
    return f"+{random.randint(1, 99)}-{random.randint(1000000000, 9999999999)}"


def random_phone_type():
    return random.choice(
        [
            example_pb2.Person.PhoneType.MOBILE,
            example_pb2.Person.PhoneType.HOME,
            example_pb2.Person.PhoneType.WORK,
        ]
    )


def create_random_person():
    person = example_pb2.Person()
    person.name = random_name()
    person.id = random.randint(1, 1000)
    person.email = random_email(person.name)

    for _ in range(random.randint(1, 3)):
        phone = person.phones.add()
        phone.number = random_phone()
        phone.type = random_phone_type()

    return person


def create_random_address_book(num_people=5):
    address_book = example_pb2.AddressBook()

    for _ in range(num_people):
        person = create_random_person()
        address_book.people.append(person)

    return address_book


if __name__ == "__main__":
    context = zmq.Context()
    socket = context.socket(zmq.PUB)

    port = "5556"
    address = f"tcp://localhost:{port}"
    topic = "test"

    socket.bind(address)
    print(f"Publishing on {address} with topic '{topic}'")

    # Publish messages in a loop
    try:
        counter = 0
        while True:
            message = create_random_address_book(num_people=10).SerializeToString()
            # Send the message with the topic prepended
            socket.send_string(topic, zmq.SNDMORE)
            socket.send_string(message)
            print(f"Sent: {counter} messages")

            counter += 1
            time.sleep(1)  # Wait 1 second between messages
    except KeyboardInterrupt:
        print("Publishing stopped.")
    finally:
        socket.close()
        context.term()
    # # Deserialize and print the AddressBook
    # with open("address_book.bin", "rb") as f:
    #     loaded_address_book = example_pb2.AddressBook()
    #     loaded_address_book.ParseFromString(f.read())
    #     print(loaded_address_book)
